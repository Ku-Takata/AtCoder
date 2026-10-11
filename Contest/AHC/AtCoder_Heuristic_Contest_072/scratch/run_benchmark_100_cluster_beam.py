import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'cluster_beam_test.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\cluster_beam'
b01_dir = r'outputs\B_01'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')

    t0 = time.time()
    with open(in_file, 'r') as fin, open(out_file, 'w') as fout:
        proc = subprocess.run([exe], stdin=fin, stdout=fout, stderr=subprocess.PIPE, text=True)
    t1 = time.time()
    elapsed = t1 - t0

    # Validate with vis.exe
    vis_proc = subprocess.run([vis_exe, in_file, out_file], capture_output=True, text=True)
    vis_score = None
    for line in vis_proc.stdout.splitlines():
        if 'Score =' in line:
            vis_score = int(line.split('=')[1].strip())

    ac = (vis_score is not None and vis_score < 100000)
    return {
        'seed': seed,
        'score': vis_score if vis_score is not None else 999999,
        'elapsed': elapsed,
        'ac': ac
    }

def main():
    print("Starting 100-case parallel benchmark for Cluster Beam Solver (方針 B: Pre-clustering + Chokudai)...")
    seeds = list(range(100))
    t0 = time.time()
    with ThreadPoolExecutor(max_workers=8) as executor:
        results = list(executor.map(run_single, seeds))
    total_time = time.time() - t0

    scores = [r['score'] for r in results if r['ac']]
    ac_count = sum(1 for r in results if r['ac'])
    total_score = sum(scores)
    avg_score = total_score / len(scores) if scores else 0

    print("\n========================================================")
    print(" BENCHMARK REPORT: Cluster Beam Solver (方針 B)")
    print("========================================================")
    print(f"Total Cases Evaluated: {len(results)}")
    print(f"Valid / Cleared (E=0): {ac_count} / {len(results)}")
    print(f"Total Score:           {total_score:,}")
    print(f"Average Score:         {avg_score:.2f}")
    print(f"Total Elapsed Time:    {total_time:.2f}s (Avg per case: {total_time/len(results)*8:.3f}s core time)")
    print("========================================================")

if __name__ == '__main__':
    main()
