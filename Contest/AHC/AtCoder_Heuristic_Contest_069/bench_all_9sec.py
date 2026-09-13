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

# --- 正規表現パターンの定義 ---
fn_score = re.compile(r"Score\s*=\s*(\d+)")
fn_time = re.compile(r"Execution Time:\s*([\d\.]+)\s*s")
fn_r = re.compile(r"R\s*=\s*([\d\.]+)")
fn_grass = re.compile(r"Map Grass:\s*(\d+)/2500")

# Phase別 実行時間
fn_p1_time = re.compile(r"Phase 1 \(Exact Rect\)\s*:\s*([\d\.]+)\s*ms")
fn_p2_time = re.compile(r"Phase 2 \(Approx Rect\)\s*:\s*([\d\.]+)\s*ms")
fn_p25_time = re.compile(r"Phase 2\.5 \(Box BFS\)\s*:\s*([\d\.]+)\s*ms")
fn_p3_time = re.compile(r"Phase 3 \(Dijkstra\)\s*:\s*([\d\.]+)\s*ms")
fn_cap_time = re.compile(r"Capacity Eval \(Cap DP\)\s*:\s*([\d\.]+)\s*ms")
fn_mm_time = re.compile(r"MultiMove Search\s*:\s*([\d\.]+)\s*ms")

# Phase別 件数 & Fee
fn_p1_info = re.compile(r"Phase 1 \(C > 0\.95\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)")
fn_p2_info = re.compile(r"Phase 2 \(C > 0\.85\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)")
fn_p25_info = re.compile(r"Phase 2\.5 \(C > 0\.75\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)")
fn_p3_info = re.compile(r"Phase 3 / MultiMove\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)")

# サイズバケット成績
fn_bucket = re.compile(r"(P=\d+-\d+):\s*acc=(\d+)\s*rej=(\d+)(?:\s*avgC=([\d\.]+))?\s*fee=(\d+)")

# 移動回数 & 移動コスト
fn_moves = re.compile(r"Moves:\s*(\d+)\s*\(Cost:\s*(\d+)\)")


def load_input_groups(in_file):
    """入力ファイルからグループ定義(P, V, T-S, Densityなど)を読み込む"""
    if not os.path.exists(in_file):
        return None, None, None, []
    with open(in_file, "r", encoding="utf-8") as f:
        lines = [line.strip() for line in f if line.strip()]
    if not lines:
        return None, None, None, []
    
    header = lines[0].split()
    n, m = int(header[0]), int(header[1])
    r_val = float(header[2])
    
    group_lines = lines[1 + n:]
    groups = []
    for g_line in group_lines:
        parts = g_line.split()
        if len(parts) >= 5:
            gid = int(parts[0])
            s = int(parts[1])
            t = int(parts[2])
            p = int(parts[3])
            v = int(parts[4])
            dur = t - s
            density = v / (p * dur) if (p * dur) > 0 else 0.0
            groups.append({
                "turn": gid, "s": s, "t": t, "p": p, "v": v,
                "dur": dur, "density": density
            })
    return n, m, r_val, groups


def parse_decisions(stdout_text):
    """標準出力からYes/No判定シーケンスを抽出"""
    return re.findall(r"\b(Yes|No)\b", stdout_text)


def run_case(seed):
    seed_str = f"{seed:04d}"
    in_file = os.path.join(IN_DIR, f"{seed_str}.txt")

    n, m, r_file, groups = load_input_groups(in_file)
    if not groups:
        return None

    cmd = f'cmd /c "{TESTER} {EXECUTABLE} < {in_file}"'
    t0 = time.time()
    res = subprocess.run(cmd, shell=True, capture_output=True, text=True, encoding="utf-8")
    wall_time = time.time() - t0

    stderr = res.stderr or ""
    stdout = res.stdout or ""
    combined = stdout + "\n" + stderr

    score = int(fn_score.search(combined).group(1)) if fn_score.search(combined) else 0
    exec_time = float(fn_time.search(stderr).group(1)) if fn_time.search(stderr) else wall_time
    r_val = float(fn_r.search(stderr).group(1)) if fn_r.search(stderr) else r_file
    grass = int(fn_grass.search(stderr).group(1)) if fn_grass.search(stderr) else 0

    p1_t = float(fn_p1_time.search(stderr).group(1)) if fn_p1_time.search(stderr) else 0.0
    p2_t = float(fn_p2_time.search(stderr).group(1)) if fn_p2_time.search(stderr) else 0.0
    p25_t = float(fn_p25_time.search(stderr).group(1)) if fn_p25_time.search(stderr) else 0.0
    p3_t = float(fn_p3_time.search(stderr).group(1)) if fn_p3_time.search(stderr) else 0.0
    cap_t = float(fn_cap_time.search(stderr).group(1)) if fn_cap_time.search(stderr) else 0.0
    mm_t = float(fn_mm_time.search(stderr).group(1)) if fn_mm_time.search(stderr) else 0.0

    m1 = fn_p1_info.search(stderr)
    cnt_p1, fee_p1 = (int(m1.group(1)), int(m1.group(2))) if m1 else (0, 0)
    m2 = fn_p2_info.search(stderr)
    cnt_p2, fee_p2 = (int(m2.group(1)), int(m2.group(2))) if m2 else (0, 0)
    m25 = fn_p25_info.search(stderr)
    cnt_p25, fee_p25 = (int(m25.group(1)), int(m25.group(2))) if m25 else (0, 0)
    m3 = fn_p3_info.search(stderr)
    cnt_p3, fee_p3 = (int(m3.group(1)), int(m3.group(2))) if m3 else (0, 0)

    buckets = {}
    for match in fn_bucket.finditer(stderr):
        b_name = match.group(1)
        acc = int(match.group(2))
        rej = int(match.group(3))
        avg_c = float(match.group(4)) if match.group(4) else 0.0
        fee = int(match.group(5))
        buckets[b_name] = {"acc": acc, "rej": rej, "avg_c": avg_c, "fee": fee}

    m_move = fn_moves.search(stderr)
    moves_cnt = int(m_move.group(1)) if m_move else 0
    moves_cost = int(m_move.group(2)) if m_move else 0

    # Yes/No 判定を各グループと紐付け
    decisions = parse_decisions(stdout)
    accepted_groups = []
    rejected_groups = []

    if len(decisions) == len(groups):
        for idx, dec in enumerate(decisions):
            g = groups[idx]
            if dec == "Yes":
                accepted_groups.append(g)
            else:
                rejected_groups.append(g)

    return {
        "seed": seed,
        "score": score,
        "time": exec_time,
        "r": r_val,
        "grass": grass,
        "p1_t": p1_t, "p2_t": p2_t, "p25_t": p25_t, "p3_t": p3_t, "cap_t": cap_t, "mm_t": mm_t,
        "cnt_p1": cnt_p1, "fee_p1": fee_p1,
        "cnt_p2": cnt_p2, "fee_p2": fee_p2,
        "cnt_p25": cnt_p25, "fee_p25": fee_p25,
        "cnt_p3": cnt_p3, "fee_p3": fee_p3,
        "buckets": buckets,
        "moves_cnt": moves_cnt,
        "moves_cost": moves_cost,
        "accepted_groups": accepted_groups,
        "rejected_groups": rejected_groups,
    }


def mean(lst):
    return sum(lst) / len(lst) if lst else 0.0

def median(lst):
    s = sorted(lst)
    n = len(s)
    if n == 0: return 0.0
    return s[n // 2] if n % 2 == 1 else (s[n // 2 - 1] + s[n // 2]) / 2.0

def stdev(lst):
    m = mean(lst)
    return math.sqrt(sum((x - m) ** 2 for x in lst) / len(lst)) if len(lst) > 1 else 0.0

def correlation(x, y):
    n = len(x)
    if n <= 1: return 0.0
    mx, my = mean(x), mean(y)
    cov = sum((x[i] - mx) * (y[i] - my) for i in range(n))
    vx = sum((x[i] - mx) ** 2 for i in range(n))
    vy = sum((y[i] - my) ** 2 for i in range(n))
    if vx == 0 or vy == 0: return 0.0
    return cov / math.sqrt(vx * vy)


def main():
    print(f"Executing and Analyzing {SEEDS_COUNT} seeds in parallel...")
    results = []

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = [executor.submit(run_case, seed) for seed in range(SEEDS_COUNT)]
        for f in concurrent.futures.as_completed(futures):
            res = f.result()
            if res:
                results.append(res)

    results.sort(key=lambda x: x["seed"])
    total_cases = len(results)
    if total_cases == 0:
        print("No results found.")
        return

    times = [r["time"] for r in results]
    scores = [r["score"] for r in results]
    r_vals = [r["r"] for r in results]
    grass_vals = [r["grass"] for r in results]

    # 集計計算
    sum_score = sum(scores)
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

    # グループ属性の全体統合
    all_acc_groups = [g for r in results for g in r["accepted_groups"]]
    all_rej_groups = [g for r in results for g in r["rejected_groups"]]

    print("\n==================================================")
    print("   AHC069 COMPREHENSIVE BENCHMARK REPORT")
    print(f"   ({total_cases} cases aggregated)")
    print("==================================================")
    print(f"Average Total Score : {sum_score / total_cases:,.2f}")
    print(f"Average Total Fee   : {sum_total_fee / total_cases:,.2f}")
    print(f"Average Accept Count: {sum_total_cnt / total_cases:.1f} groups/case")

    print("\n--------------------------------------------------")
    print("   1. EXECUTION TIME OVERVIEW & PERCENTILES")
    print("--------------------------------------------------")
    print(f"Average Time : {mean(times):.3f} s  | Median Time : {median(times):.3f} s")
    print(f"Min Time     : {min(times):.3f} s  | Max Time    : {max(times):.3f} s (StdDev: {stdev(times):.3f}s)")
    
    st = sorted(times)
    p25, p75, p90, p95, p99 = st[int(len(st)*0.25)], st[int(len(st)*0.75)], st[int(len(st)*0.90)], st[int(len(st)*0.95)], st[int(len(st)*0.99)]
    print(f"Percentiles  : Q1(25%)={p25:.3f}s | Q2(50%)={median(times):.3f}s | Q3(75%)={p75:.3f}s | 90%={p90:.3f}s | 95%={p95:.3f}s | 99%={p99:.3f}s")

    print("\n--------------------------------------------------")
    print("   2. EXECUTION TIME HISTOGRAM")
    print("--------------------------------------------------")
    # 9秒制限用に分布バケットを調整
    buckets_time = [
        ("0.0s - 2.0s", 0.0, 2.0),
        ("2.0s - 4.0s", 2.0, 4.0),
        ("4.0s - 6.0s", 4.0, 6.0),
        ("6.0s - 7.5s", 7.0, 7.5),
        ("7.5s - 8.5s", 7.5, 8.5),
        ("8.5s - 8.8s", 8.5, 8.8),
        ("8.8s - 9.5s (Guard)", 8.8, 9.5)
    ]
    for label, low, high in buckets_time:
        cnt = sum(1 for t in times if low <= t < high)
        pct = (cnt / total_cases) * 100.0
        bar = "#" * int(pct / 2)
        print(f"  {label:<25} : {cnt:4d} cases ({pct:5.1f}%) | {bar}")

    print("\n--------------------------------------------------")
    print("   3. CORRELATION ANALYSIS (What impacts performance?)")
    print("--------------------------------------------------")
    print(f"  Corr(Move Cost R, Exec Time) : {correlation(r_vals, times):+.3f}")
    print(f"  Corr(Map Grass, Exec Time)   : {correlation(grass_vals, times):+.3f}  {'[STRONG]' if abs(correlation(grass_vals, times)) > 0.4 else ''}")
    print(f"  Corr(Score, Exec Time)       : {correlation(scores, times):+.3f}")
    print(f"  Corr(Map Grass, Score)       : {correlation(grass_vals, scores):+.3f}")

    print("\n--------------------------------------------------")
    print("   4. PHASE BREAKDOWN (Time & Fee Contribution)")
    print("--------------------------------------------------")
    fee_safe = max(1, sum_total_fee)
    phases = [
        ("Phase 1 (C > 0.95)", sum_fee_p1, sum_cnt_p1, mean([r["p1_t"] for r in results])),
        ("Phase 2 (C > 0.85)", sum_fee_p2, sum_cnt_p2, mean([r["p2_t"] for r in results])),
        ("Phase 2.5 (C > 0.75)", sum_fee_p25, sum_cnt_p25, mean([r["p25_t"] for r in results])),
        ("Phase 3 / MultiMove", sum_fee_p3, sum_cnt_p3, mean([r["p3_t"] for r in results])),
    ]
    print(f"{'Phase Name':<20} | {'Avg Time':<9} | {'Total Fee':<14} | {'Share (%)':<8} | {'Avg Count':<9}")
    print("-" * 75)
    for name, fee, cnt, avg_t in phases:
        pct = (fee / fee_safe) * 100.0
        print(f"{name:<20} | {avg_t:6.2f} ms | {fee:14,} | {pct:7.2f}% | {cnt / total_cases:8.1f}")
    
    cap_avg_t = mean([r["cap_t"] for r in results])
    mm_avg_t = mean([r["mm_t"] for r in results])
    print("-" * 75)
    print(f"  * Capacity Eval DP Avg Time : {cap_avg_t:.2f} ms")
    print(f"  * MultiMove Search Avg Time : {mm_avg_t:.2f} ms")

    print("\n--------------------------------------------------")
    print("   5. SIZE BUCKET ANALYSIS (Group Size P_i Performance)")
    print("--------------------------------------------------")
    bucket_names = ["P=4-30", "P=31-70", "P=71-110", "P=111-150"]
    print(f"{'Bucket Name':<12} | {'Total Acc':<10} | {'Total Rej':<10} | {'Acc Rate':<8} | {'Avg Compact':<11} | {'Total Fee':<14}")
    print("-" * 75)
    for b_name in bucket_names:
        tot_acc = sum(r["buckets"].get(b_name, {}).get("acc", 0) for r in results)
        tot_rej = sum(r["buckets"].get(b_name, {}).get("rej", 0) for r in results)
        tot_fee = sum(r["buckets"].get(b_name, {}).get("fee", 0) for r in results)
        
        c_list = [r["buckets"][b_name]["avg_c"] for r in results if b_name in r["buckets"] and r["buckets"][b_name]["acc"] > 0]
        avg_c = mean(c_list) if c_list else 0.0
        
        req = tot_acc + tot_rej
        acc_rate = (tot_acc / req * 100.0) if req > 0 else 0.0
        print(f"{b_name:<12} | {tot_acc:10,} | {tot_rej:10,} | {acc_rate:6.1f}%  | {avg_c:11.4f} | {tot_fee:14,}")

    print("\n--------------------------------------------------")
    print("   6. MOVE & COST BREAKDOWN")
    print("--------------------------------------------------")
    tot_moves = sum(r["moves_cnt"] for r in results)
    tot_move_cost = sum(r["moves_cost"] for r in results)
    print(f"Total MultiMoves Executed : {tot_moves:,}  (Avg {tot_moves / total_cases:.2f} moves/case)")
    print(f"Total Move Cost Incurred  : {tot_move_cost:,}  (Avg {tot_move_cost / total_cases:,.1f} fee/case)")
    print(f"Net Fee (Total Fee - Cost): {sum_total_fee - tot_move_cost:,}")

    # --- 新設セクション: 拒否されたグループの詳細分析 (WHY & WHAT) ---
    print("\n==================================================")
    print("   7. DETAILED REJECTED GROUPS PROFILE (WHY & WHAT)")
    print("==================================================")
    if all_rej_groups:
        tot_rej_cnt = len(all_rej_groups)
        tot_acc_cnt = len(all_acc_groups)
        tot_all_cnt = tot_rej_cnt + tot_acc_cnt

        sum_lost_fee = sum(g["v"] for g in all_rej_groups)
        sum_acc_potential = sum(g["v"] for g in all_acc_groups)
        sum_total_potential = sum_lost_fee + sum_acc_potential

        print(f"Total Rejected Groups : {tot_rej_cnt:,} / {tot_all_cnt:,} ({tot_rej_cnt / tot_all_cnt * 100.0:.1f}%)")
        print(f"Total Potential Fee Lost: {sum_lost_fee:,} ({(sum_lost_fee / max(1, sum_total_potential)) * 100.0:.1f}% of Potential)")
        print(f"Avg Lost Fee / Case     : {sum_lost_fee / total_cases:,.1f}")

        print("\n--- A. Attribute Comparison: Accepted vs Rejected ---")
        print(f"{'Metric':<25} | {'Accepted Groups':<18} | {'Rejected Groups':<18}")
        print("-" * 68)
        print(f"{'Average Size (P)':<25} | {mean([g['p'] for g in all_acc_groups]):18.1f} | {mean([g['p'] for g in all_rej_groups]):18.1f}")
        print(f"{'Average Value (V)':<25} | {mean([g['v'] for g in all_acc_groups]):18,.1f} | {mean([g['v'] for g in all_rej_groups]):18,.1f}")
        print(f"{'Average Duration (T-S)':<25} | {mean([g['dur'] for g in all_acc_groups]):18.1f} | {mean([g['dur'] for g in all_rej_groups]):18.1f}")
        print(f"{'Average Density V/(P*Dur)':<25} | {mean([g['density'] for g in all_acc_groups]):18.3f} | {mean([g['density'] for g in all_rej_groups]):18.3f}")

        print("\n--- B. Rejection Timing Breakdown (Arrival Turn) ---")
        early_rej = [g for g in all_rej_groups if g["turn"] < 333]
        mid_rej   = [g for g in all_rej_groups if 333 <= g["turn"] < 666]
        late_rej  = [g for g in all_rej_groups if g["turn"] >= 666]

        print(f"{'Arrival Phase':<20} | {'Rej Count':<12} | {'Share (%)':<10} | {'Avg Value (V)':<15} | {'Total Lost Fee':<16}")
        print("-" * 80)
        for p_name, r_list in [("Early (Turns 0-332)", early_rej), ("Mid (Turns 333-665)", mid_rej), ("Late (Turns 666-999)", late_rej)]:
            cnt = len(r_list)
            pct = (cnt / tot_rej_cnt * 100.0) if tot_rej_cnt > 0 else 0.0
            avg_v = mean([g["v"] for g in r_list])
            sum_v = sum(g["v"] for g in r_list)
            print(f"{p_name:<20} | {cnt:12,} | {pct:9.1f}% | {avg_v:15,.1f} | {sum_v:16,}")

        print("\n--- C. Rejection Details by Size Bucket (P) ---")
        print(f"{'Bucket Name':<12} | {'Rej Count':<10} | {'Rej Rate':<9} | {'Avg Rej Value':<15} | {'Total Lost Fee':<16}")
        print("-" * 70)
        p_ranges = [("P=4-30", 4, 30), ("P=31-70", 31, 70), ("P=71-110", 71, 110), ("P=111-150", 111, 150)]
        for b_name, p_min, p_max in p_ranges:
            b_acc = [g for g in all_acc_groups if p_min <= g["p"] <= p_max]
            b_rej = [g for g in all_rej_groups if p_min <= g["p"] <= p_max]
            b_cnt_acc = len(b_acc)
            b_cnt_rej = len(b_rej)
            b_tot = b_cnt_acc + b_cnt_rej
            rej_rate = (b_cnt_rej / b_tot * 100.0) if b_tot > 0 else 0.0
            avg_v = mean([g["v"] for g in b_rej])
            sum_v = sum(g["v"] for g in b_rej)
            print(f"{b_name:<12} | {b_cnt_rej:10,} | {rej_rate:8.1f}% | {avg_v:15,.1f} | {sum_v:16,}")

    else:
        print("  * Individual group decision tracking requires solution output (stdout).")
        print("    No detailed decision logs captured in stdout.")

    print("\n--------------------------------------------------")
    print("   8. TIME GUARD TRIGGERED SEEDS (>= 8.80s)")
    print("--------------------------------------------------")
    tg_cases = [r for r in results if r["time"] >= 8.80]
    print(f"Total Triggered Cases : {len(tg_cases)}")
    if tg_cases:
        seeds_only = [f"{r['seed']:04d}" for r in tg_cases]
        print(f"Seed Numbers ({len(seeds_only)}) : {', '.join(seeds_only)}")
        print("\nDetailed Breakdown:")
        for r in tg_cases:
            print(f"  Seed {r['seed']:04d} | Time: {r['time']:.3f} s | Score: {r['score']:10,} | Grass: {r['grass']}/2500 | R: {r['r']:.4f}")


if __name__ == "__main__":
    main()