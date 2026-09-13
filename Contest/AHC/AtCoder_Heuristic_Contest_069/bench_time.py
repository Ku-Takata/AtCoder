import concurrent.futures
import math
import os
import re
import subprocess
import time

EXECUTABLE = "./main_rs.exe"
TESTER = r"AdcJXWH4_windows\tools_x86_64-pc-windows-gnu\tester.exe"
IN_DIR = r"AdcJXWH4_windows\tools_x86_64-pc-windows-gnu\in"
SEEDS_COUNT = 1000

fn_score = re.compile(r"Score\s*=\s*(\d+)")
fn_time = re.compile(r"Execution Time:\s*([\d\.]+)\s*s")
fn_r = re.compile(r"R\s*=\s*([\d\.]+)")
fn_grass = re.compile(r"Map Grass:\s*(\d+)/2500")

fn_p1 = re.compile(r"Phase 1 \(Exact Rect\)\s*:\s*([\d\.]+)\s*ms")
fn_p2 = re.compile(r"Phase 2 \(Approx Rect\)\s*:\s*([\d\.]+)\s*ms")
fn_p3 = re.compile(r"Phase 3 \(Dijkstra\)\s*:\s*([\d\.]+)\s*ms")
fn_cap = re.compile(r"Capacity Eval \(Cap DP\)\s*:\s*([\d\.]+)\s*ms")
fn_mm = re.compile(r"MultiMove Search\s*:\s*([\d\.]+)\s*ms")


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

    m_r = fn_r.search(stderr)
    r_val = float(m_r.group(1)) if m_r else 0.0

    m_g = fn_grass.search(stderr)
    grass = int(m_g.group(1)) if m_g else 0

    p1 = float(fn_p1.search(stderr).group(1)) if fn_p1.search(stderr) else 0.0
    p2 = float(fn_p2.search(stderr).group(1)) if fn_p2.search(stderr) else 0.0
    p3 = float(fn_p3.search(stderr).group(1)) if fn_p3.search(stderr) else 0.0
    cap = (
        float(fn_cap.search(stderr).group(1))
        if fn_cap.search(stderr)
        else 0.0
    )
    mm = float(fn_mm.search(stderr).group(1)) if fn_mm.search(stderr) else 0.0

    return {
        "seed": seed,
        "score": score,
        "time": exec_time,
        "r": r_val,
        "grass": grass,
        "p1": p1,
        "p2": p2,
        "p3": p3,
        "cap": cap,
        "mm": mm,
    }


def mean(lst):
    return sum(lst) / len(lst) if lst else 0.0


def median(lst):
    s = sorted(lst)
    n = len(s)
    if n == 0:
        return 0.0
    return s[n // 2] if n % 2 == 1 else (s[n // 2 - 1] + s[n // 2]) / 2.0


def stdev(lst):
    m = mean(lst)
    return (
        math.sqrt(sum((x - m) ** 2 for x in lst) / len(lst)) if len(lst) > 1 else 0.0
    )


def correlation(x, y):
    n = len(x)
    if n <= 1:
        return 0.0
    mx, my = mean(x), mean(y)
    cov = sum((x[i] - mx) * (y[i] - my) for i in range(n))
    vx = sum((x[i] - mx) ** 2 for i in range(n))
    vy = sum((y[i] - my) ** 2 for i in range(n))
    if vx == 0 or vy == 0:
        return 0.0
    return cov / math.sqrt(vx * vy)


def main():
    print(
        f"Analyzing Execution Time & Profiling Distribution on {SEEDS_COUNT} seeds..."
    )
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

    times = [r["time"] for r in results]
    scores = [r["score"] for r in results]
    r_vals = [r["r"] for r in results]
    grass_vals = [r["grass"] for r in results]

    print("\n==================================================")
    print("   1. EXECUTION TIME STATISTICAL OVERVIEW")
    print("==================================================")
    print(f"Total Cases  : {len(results)}")
    print(f"Average Time : {mean(times):.3f} s")
    print(f"Median Time  : {median(times):.3f} s")
    print(f"Min Time     : {min(times):.3f} s")
    print(f"Max Time     : {max(times):.3f} s")
    print(f"Std Dev      : {stdev(times):.3f} s")

    # パーセンタイル (分位数)
    st = sorted(times)
    p25 = st[int(len(st) * 0.25)]
    p50 = median(times)
    p75 = st[int(len(st) * 0.75)]
    p90 = st[int(len(st) * 0.90)]
    p95 = st[int(len(st) * 0.95)]
    p99 = st[int(len(st) * 0.99)]

    print("\n--- Percentiles (Quantiles) ---")
    print(f"  25th Percentile (Q1) : {p25:.3f} s")
    print(f"  50th Percentile (Q2) : {p50:.3f} s  (Median)")
    print(f"  75th Percentile (Q3) : {p75:.3f} s")
    print(f"  90th Percentile      : {p90:.3f} s")
    print(f"  95th Percentile      : {p95:.3f} s")
    print(f"  99th Percentile      : {p99:.3f} s")

    print("\n==================================================")
    print("   2. EXECUTION TIME HISTOGRAM (0.2s Uniform Buckets)")
    print("==================================================")
    buckets = [
        ("0.0s - 0.2s", 0.0, 0.2),
        ("0.2s - 0.4s", 0.2, 0.4),
        ("0.4s - 0.6s", 0.4, 0.6),
        ("0.6s - 0.8s", 0.6, 0.8),
        ("0.8s - 1.0s", 0.8, 1.0),
        ("1.0s - 1.2s", 1.0, 1.2),
        ("1.2s - 1.4s", 1.2, 1.4),
        ("1.4s - 1.6s", 1.4, 1.6),
        ("1.6s - 1.82s", 1.6, 1.82),
        ("1.82s - 2.05s (Time Guard)", 1.82, 2.05),
    ]

    for label, low, high in buckets:
        cnt = sum(1 for t in times if low <= t < high)
        pct = (cnt / len(times)) * 100.0
        bar = "#" * int(pct / 2)
        print(f"  {label:<27} : {cnt:4d} cases ({pct:5.1f}%) | {bar}")

    print("\n==================================================")
    print("   3. CORRELATION ANALYSIS (What causes slowness?)")
    print("==================================================")
    corr_r = correlation(r_vals, times)
    corr_grass = correlation(grass_vals, times)
    corr_score = correlation(scores, times)

    print(
        f"  Corr(Move Cost R, Exec Time) : {corr_r:+.3f}  {'[Strong]' if abs(corr_r) > 0.4 else '[Moderate/Weak]'}"
    )
    print(
        f"  Corr(Map Grass, Exec Time)   : {corr_grass:+.3f}  {'[Strong]' if abs(corr_grass) > 0.4 else '[Moderate/Weak]'}"
    )
    print(
        f"  Corr(Score, Exec Time)       : {corr_score:+.3f}  {'[Strong]' if abs(corr_score) > 0.4 else '[Moderate/Weak]'}"
    )

    print("\n==================================================")
    print("   4. PHASE TIME BREAKDOWN (Average per Case)")
    print("==================================================")
    avg_p1 = mean([r["p1"] for r in results])
    avg_p2 = mean([r["p2"] for r in results])
    avg_p3 = mean([r["p3"] for r in results])
    avg_cap = mean([r["cap"] for r in results])
    avg_mm = mean([r["mm"] for r in results])

    print(f"  Phase 1 (Exact Rect)   : {avg_p1:6.2f} ms")
    print(f"  Phase 2 (Approx Rect)  : {avg_p2:6.2f} ms")
    print(f"  Phase 3 (Dijkstra)     : {avg_p3:6.2f} ms")
    print(f"  Capacity Eval (Cap DP) : {avg_cap:6.2f} ms")
    print(f"  MultiMove Search       : {avg_mm:6.2f} ms")

    # 新規追加: Time Guard (1.82s 以上) に到達したシード値一覧の表示
    print("\n==================================================")
    print("   5. TIME GUARD TRIGGERED SEEDS (>= 1.82s)")
    print("==================================================")
    tg_cases = [r for r in results if r["time"] >= 1.82]
    print(f"Total Triggered Cases : {len(tg_cases)}")
    if tg_cases:
        seeds_only = [f"{r['seed']:04d}" for r in tg_cases]
        print(f"Seed Numbers ({len(seeds_only)}) : {', '.join(seeds_only)}")
        print("\nDetailed Breakdown:")
        for r in tg_cases:
            print(
                f"  Seed {r['seed']:04d} | Time: {r['time']:.3f} s | Score: {r['score']:10,} | Grass: {r['grass']}/2500 | R: {r['r']:.4f}"
            )


if __name__ == "__main__":
    main()