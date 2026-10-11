import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_07_A2.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_07_A2'
b07_dir = r'outputs\B_07'
b06_dir = r'outputs\B_06'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b07_out_file = os.path.join(b07_dir, f'{seed}.txt')
    b06_out_file = os.path.join(b06_dir, f'{seed}.txt')

    t0 = time.time()
    timed_out = False
    try:
        with open(in_file, 'r') as fin, open(out_file, 'w') as fout:
            proc = subprocess.run([exe], stdin=fin, stdout=fout, stderr=subprocess.PIPE, text=True, timeout=2.5)
    except subprocess.TimeoutExpired:
        timed_out = True
    t1 = time.time()
    elapsed = t1 - t0

    # Validate with vis.exe
    score = 0
    if not timed_out and os.path.exists(out_file):
        vis_proc = subprocess.run([vis_exe, in_file, out_file], capture_output=True, text=True)
        for line in vis_proc.stdout.splitlines():
            if 'Score =' in line:
                score = int(line.split('=')[1].strip())

    # Read b07 score
    b07_score = None
    if os.path.exists(b07_out_file):
        vis_b07 = subprocess.run([vis_exe, in_file, b07_out_file], capture_output=True, text=True)
        for line in vis_b07.stdout.splitlines():
            if 'Score =' in line:
                b07_score = int(line.split('=')[1].strip())

    # Read b06 score
    b06_score = None
    if os.path.exists(b06_out_file):
        vis_b06 = subprocess.run([vis_exe, in_file, b06_out_file], capture_output=True, text=True)
        for line in vis_b06.stdout.splitlines():
            if 'Score =' in line:
                b06_score = int(line.split('=')[1].strip())

    diff_b07 = score - b07_score if b07_score is not None else 0
    diff_b06 = score - b06_score if b06_score is not None else 0
    return {
        'seed': seed,
        'score': score,
        'b07_score': b07_score,
        'diff_b07': diff_b07,
        'b06_score': b06_score,
        'diff_b06': diff_b06,
        'elapsed': elapsed,
        'timed_out': timed_out,
    }

def main():
    t_start = time.time()
    seeds = list(range(100))
    results = []

    with ThreadPoolExecutor(max_workers=4) as pool:
        for res in pool.map(run_single, seeds):
            results.append(res)
            if len(results) % 20 == 0:
                print(f"Done {len(results)}/100 tests...")

    results.sort(key=lambda x: x['seed'])

    total_score = sum(r['score'] for r in results)
    total_b07 = sum(r['b07_score'] for r in results if r['b07_score'] is not None)
    total_diff_b07 = total_score - total_b07

    total_b06 = sum(r['b06_score'] for r in results if r['b06_score'] is not None)
    total_diff_b06 = total_score - total_b06

    wins_b07 = sum(1 for r in results if r['diff_b07'] < 0)
    ties_b07 = sum(1 for r in results if r['diff_b07'] == 0)
    losses_b07 = sum(1 for r in results if r['diff_b07'] > 0)

    wins_b06 = sum(1 for r in results if r['diff_b06'] < 0)
    ties_b06 = sum(1 for r in results if r['diff_b06'] == 0)
    losses_b06 = sum(1 for r in results if r['diff_b06'] > 0)

    zero_scores = sum(1 for r in results if r['score'] == 0)
    timeouts = sum(1 for r in results if r['timed_out'] or r['elapsed'] >= 2.05)
    max_elapsed = max(r['elapsed'] for r in results)

    print("\n" + "="*55)
    print(f"BENCHMARK RESULTS: B_07_A2 (Zero-Alloc & Guided SA) vs B_07")
    print("="*55)
    print(f"Total Score (B_07_A2): {total_score} ops")
    print(f"Total Score (B_07)   : {total_b07} ops")
    print(f"Score Diff (vs B_07) : {total_diff_b07:+d} ops ({total_diff_b07/total_b07*100:+.2f}%)")
    print(f"Win/Tie/Loss (vs B07): {wins_b07} W / {ties_b07} T / {losses_b07} L")
    print(f"Score Diff (vs B_06) : {total_diff_b06:+d} ops ({total_diff_b06/total_b06*100:+.2f}%)")
    print(f"Win/Tie/Loss (vs B06): {wins_b06} W / {ties_b06} T / {losses_b06} L")
    print(f"Invalid (Score=0)    : {zero_scores}")
    print(f"TLE Cases (>= 2.05s) : {timeouts}")
    print(f"Max Elapsed Time     : {max_elapsed:.2f} s")
    print(f"Total Benchmark Time : {time.time() - t_start:.2f} s")
    print("="*55)

if __name__ == '__main__':
    main()
