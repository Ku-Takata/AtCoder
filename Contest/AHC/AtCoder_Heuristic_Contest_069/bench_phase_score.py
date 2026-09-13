import concurrent.futures
import os
import re
import subprocess
import time

EXECUTABLE = "./main_rs.exe"
TESTER = r"AdcJXWH4_windows\tools_x86_64-pc-windows-gnu\tester.exe"
IN_DIR = r"AdcJXWH4_windows\tools_x86_64-pc-windows-gnu\in"
SEEDS_COUNT = 1000

# 正規表現パターン
fn_score = re.compile(r"Score\s*=\s*(\d+)")
fn_time = re.compile(r"Execution Time:\s*([\d\.]+)\s*s")

fn_p1_info = re.compile(
    r"Phase 1 \(C > 0\.95\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)"
)
fn_p2_info = re.compile(
    r"Phase 2 \(C > 0\.85\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)"
)
fn_p25_info = re.compile(
    r"Phase 2\.5 \(C > 0\.75\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)"
)
fn_p3_info = re.compile(
    r"Phase 3 / MultiMove\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)"
)


def run_case(seed):
    seed_str = f"{seed:04d}"
    in_file = os.path.join(IN_DIR, f"{seed_str}.txt")

    if not os.path.exists(in_file):
        return None

    cmd = f'cmd /c "{TESTER} {EXECUTABLE} < {in_file}"'
    t0 = time.time()
    res = subprocess.run(
        cmd, shell=True, capture_output=True, text=True, encoding="utf-8"
    )
    wall_time = time.time() - t0

    stderr = res.stderr or ""
    stdout = res.stdout or ""
    combined = stdout + "\n" + stderr

    m_score = fn_score.search(combined)
    score = int(m_score.group(1)) if m_score else 0

    m_time = fn_time.search(stderr)
    exec_time = float(m_time.group(1)) if m_time else wall_time

    # Phase情報抽出 (件数, 金額)
    m1 = fn_p1_info.search(stderr)
    cnt_p1, fee_p1 = (int(m1.group(1)), int(m1.group(2))) if m1 else (0, 0)

    m2 = fn_p2_info.search(stderr)
    cnt_p2, fee_p2 = (int(m2.group(1)), int(m2.group(2))) if m2 else (0, 0)

    m25 = fn_p25_info.search(stderr)
    cnt_p25, fee_p25 = (
        (int(m25.group(1)), int(m25.group(2))) if m25 else (0, 0)
    )

    m3 = fn_p3_info.search(stderr)
    cnt_p3, fee_p3 = (int(m3.group(1)), int(m3.group(2))) if m3 else (0, 0)

    return {
        "seed": seed,
        "score": score,
        "time": exec_time,
        "cnt_p1": cnt_p1,
        "fee_p1": fee_p1,
        "cnt_p2": cnt_p2,
        "fee_p2": fee_p2,
        "cnt_p25": cnt_p25,
        "fee_p25": fee_p25,
        "cnt_p3": cnt_p3,
        "fee_p3": fee_p3,
    }


def main():
    print(f"Analyzing Phase-wise Score Breakdown across {SEEDS_COUNT} seeds...")
    results = []

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = [
            executor.submit(run_case, seed) for seed in range(SEEDS_COUNT)
        ]
        for f in concurrent.futures.as_completed(futures):
            res = f.result()
            if res:
                results.append(res)

    results.sort(key=lambda x: x["seed"])

    total_cases = len(results)
    if total_cases == 0:
        print("No cases found.")
        return

    # 全体集計
    sum_score = sum(r["score"] for r in results)
    sum_fee_p1 = sum(r["fee_p1"] for r in results)
    sum_fee_p2 = sum(r["fee_p2"] for r in results)
    sum_fee_p25 = sum(r["fee_p25"] for r in results)
    sum_fee_p3 = sum(r["fee_p3"] for r in results)
    sum_total_fee = sum_fee_p1 + sum_fee_p2 + sum_fee_p25 + sum_fee_p3

    sum_cnt_p1 = sum(r["cnt_p1"] for r in results)
    sum_cnt_p2 = sum(r["cnt_p2"] for r in results)
    sum_cnt_p25 = sum(r["cnt_p25"] for r in results)
    sum_cnt_p3 = sum(r["cnt_p3"] for r in results)
    sum_total_cnt = sum_cnt_p1 + sum_cnt_p2 + sum_cnt_p25 + sum_cnt_p3

    fee_safe = max(1, sum_total_fee)

    print("\n==================================================")
    print(f"   PHASE-WISE SCORE (FEE) STATISTICAL REPORT")
    print(f"   ({total_cases} cases aggregated)")
    print("==================================================")
    print(f"Average Total Score : {sum_score / total_cases:,.2f}")
    print(f"Average Total Fee   : {sum_total_fee / total_cases:,.2f}")
    print(f"Average Accept Count: {sum_total_cnt / total_cases:.1f} groups/case")

    print("\n--------------------------------------------------")
    print("   1. OVERALL PHASE CONTRIBUTION (1000 Cases Combined)")
    print("--------------------------------------------------")
    print(
        f"{'Phase Name':<22} | {'Total Fee':<15} | {'Share (%)':<8} | {'Avg Fee/Case':<14} | {'Avg Count':<10}"
    )
    print("-" * 80)

    phases = [
        ("Phase 1 (C > 0.95)", sum_fee_p1, sum_cnt_p1),
        ("Phase 2 (C > 0.85)", sum_fee_p2, sum_cnt_p2),
        ("Phase 2.5 (C > 0.75)", sum_fee_p25, sum_cnt_p25),
        ("Phase 3 / MultiMove", sum_fee_p3, sum_cnt_p3),
    ]

    for name, fee, cnt in phases:
        pct = (fee / fee_safe) * 100.0
        avg_fee = fee / total_cases
        avg_cnt = cnt / total_cases
        print(
            f"{name:<22} | {fee:15,} | {pct:7.2f}% | {avg_fee:14,f} | {avg_cnt:9.1f}"
        )

    print("-" * 80)
    print(
        f"{'TOTAL':<22} | {sum_total_fee:15,} | {100.0:7.2f}% | {sum_total_fee / total_cases:14,.1f} | {sum_total_cnt / total_cases:9.1f}"
    )

    print("\n--------------------------------------------------")
    print("   2. VISUAL SHARE BREAKDOWN (Bar Chart)")
    print("--------------------------------------------------")
    for name, fee, _ in phases:
        pct = (fee / fee_safe) * 100.0
        bar = "#" * int(pct / 2)
        print(f"  {name:<22} : {pct:5.1f}% | {bar}")


if __name__ == "__main__":
    main()