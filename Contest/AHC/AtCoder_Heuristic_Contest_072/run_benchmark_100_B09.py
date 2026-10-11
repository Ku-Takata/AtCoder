import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_09.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_09'
b08_dir = r'outputs\B_08'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b08_out_file = os.path.join(b08_dir, f'{seed}.txt')

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

    # Read b08 score
    b08_score = None
    if os.path.exists(b08_out_file):
        vis_b08 = subprocess.run([vis_exe, in_file, b08_out_file], capture_output=True, text=True)
        for line in vis_b08.stdout.splitlines():
            if 'Score =' in line:
                b08_score = int(line.split('=')[1].strip())

    # Parse initial slimes
    initial_slimes = 0
    with open(in_file, 'r') as fin:
        for line in fin.readlines()[1:]:
            for ch in line:
                if ch.islower():
                    initial_slimes += 1

    diff = score - b08_score if b08_score is not None else 0
    return {
        'seed': seed,
        'score': score,
        'b08_score': b08_score,
        'diff_b08': diff,
        'elapsed': elapsed,
        'timed_out': timed_out,
        'initial_slimes': initial_slimes,
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
    total_b08 = sum(r['b08_score'] for r in results if r['b08_score'] is not None)
    total_diff = total_score - total_b08

    wins = sum(1 for r in results if r['diff_b08'] < 0)
    ties = sum(1 for r in results if r['diff_b08'] == 0)
    losses = sum(1 for r in results if r['diff_b08'] > 0)
    zero_scores = sum(1 for r in results if r['score'] == 0)
    timeouts = sum(1 for r in results if r['timed_out'] or r['elapsed'] >= 2.05)
    max_elapsed = max(r['elapsed'] for r in results)

    print("\n" + "="*55)
    print(f"BENCHMARK RESULTS: B_09 (Adaptive LNS) vs B_08")
    print("="*55)
    print(f"Total Score (B_09)             : {total_score} ops")
    print(f"Total Score (B_08)             : {total_b08} ops")
    print(f"Score Difference               : {total_diff:+d} ops ({total_diff/total_b08*100:+.2f}%)")
    print(f"Win / Tie / Loss               : {wins} W / {ties} T / {losses} L")
    print(f"Invalid (Score=0)              : {zero_scores}")
    print(f"TLE Cases (>= 2.05s)           : {timeouts}")
    print(f"Max Elapsed Time               : {max_elapsed:.2f} s")
    print(f"Total Benchmark Time           : {time.time() - t_start:.2f} s")
    print("="*55)

    # Top improvements
    results_with_diff = [r for r in results if r['diff_b08'] != 0]
    results_with_diff.sort(key=lambda x: x['diff_b08'])
    print("\nTop 5 Improved Seeds:")
    for r in results_with_diff[:5]:
        print(f"  Seed {r['seed']}: {r['b08_score']} -> {r['score']} ({r['diff_b08']:+d}) [Slimes: {r['initial_slimes']}]")

    print("\nTop 5 Degraded Seeds:")
    for r in results_with_diff[-5:][::-1]:
        print(f"  Seed {r['seed']}: {r['b08_score']} -> {r['score']} ({r['diff_b08']:+d}) [Slimes: {r['initial_slimes']}]")

if __name__ == '__main__':
    main()
