import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_03_soft_C.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_03_soft_C'
b03_dir = r'outputs\B_03'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b03_out_file = os.path.join(b03_dir, f'{seed}.txt')

    t0 = time.time()
    with open(in_file, 'r') as fin, open(out_file, 'w') as fout:
        proc = subprocess.run([exe], stdin=fin, stdout=fout, stderr=subprocess.PIPE, text=True)
    t1 = time.time()
    elapsed = t1 - t0

    # Validate with vis.exe
    vis_proc = subprocess.run([vis_exe, in_file, out_file], capture_output=True, text=True)
    score = 0
    for line in vis_proc.stdout.splitlines():
        if 'Score =' in line:
            score = int(line.split('=')[1].strip())

    # Read b03 score
    b03_score = None
    if os.path.exists(b03_out_file):
        v = subprocess.run([vis_exe, in_file, b03_out_file], capture_output=True, text=True)
        for line in v.stdout.splitlines():
            if 'Score =' in line:
                b03_score = int(line.split('=')[1].strip())

    # Count initial slimes
    with open(in_file, 'r') as f:
        in_lines = [l.strip() for l in f if l.strip()]
    initial_slimes = sum(c.islower() for l in in_lines[1:] for c in l)

    return {
        'seed': seed,
        'seed_int': seed_int,
        'initial_slimes': initial_slimes,
        'score': score,
        'b03_score': b03_score,
        'diff_b03': score - b03_score if b03_score is not None else 0,
        'elapsed': elapsed
    }

def main():
    print(f"Starting benchmark of 100 seeds with {exe} (parallel=8)...")
    t_start = time.time()
    results = []
    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = [executor.submit(run_single, i) for i in range(100)]
        for f in futures:
            results.append(f.result())

    results.sort(key=lambda x: x['seed_int'])
    t_total = time.time() - t_start

    total_score = sum(r['score'] for r in results)
    total_b03 = sum(r['b03_score'] for r in results if r['b03_score'] is not None)
    total_diff = total_score - total_b03
    unreturned_cases = [r['seed'] for r in results if r['score'] >= 100000]
    improved = [r for r in results if r['diff_b03'] < 0]
    degraded = [r for r in results if r['diff_b03'] > 0]
    tied = [r for r in results if r['diff_b03'] == 0]

    # Tier statistics
    small = [r for r in results if r['initial_slimes'] <= 75]
    med = [r for r in results if 75 < r['initial_slimes'] <= 115]
    dense = [r for r in results if r['initial_slimes'] > 115]

    print("\n" + "="*60)
    print("BENCHMARK SUMMARY (B_03_soft_C vs B_03 baseline 33,098)")
    print("="*60)
    print(f"Total Score (moves): {total_score:,} (B_03 baseline: {total_b03:,})")
    print(f"Overall Difference : {total_diff:+d} moves ({'IMPROVEMENT!' if total_diff < 0 else 'DEGRADATION'})")
    print(f"Total time         : {t_total:.1f}s")
    print(f"Unreturned cases   : {len(unreturned_cases)}")
    print(f"Win/Tie/Loss       : {len(improved)} wins / {len(tied)} ties / {len(degraded)} losses")
    print("-" * 60)
    print(f"Small Tier  (<=75) : {sum(r['score'] for r in small):,} vs {sum(r['b03_score'] for r in small):,} ({sum(r['score'] for r in small)-sum(r['b03_score'] for r in small):+d}) [n={len(small)}]")
    print(f"Medium Tier (76-115): {sum(r['score'] for r in med):,} vs {sum(r['b03_score'] for r in med):,} ({sum(r['score'] for r in med)-sum(r['b03_score'] for r in med):+d}) [n={len(med)}]")
    print(f"Dense Tier  (>115) : {sum(r['score'] for r in dense):,} vs {sum(r['b03_score'] for r in dense):,} ({sum(r['score'] for r in dense)-sum(r['b03_score'] for r in dense):+d}) [n={len(dense)}]")
    print("-" * 60)

    if improved:
        print("\nTop 5 Improvements:")
        for r in sorted(improved, key=lambda x: x['diff_b03'])[:5]:
            print(f"  Seed {r['seed']} (slimes={r['initial_slimes']}): {r['b03_score']} -> {r['score']} ({r['diff_b03']:+d})")
    if degraded:
        print("\nTop 5 Degradations:")
        for r in sorted(degraded, key=lambda x: x['diff_b03'], reverse=True)[:5]:
            print(f"  Seed {r['seed']} (slimes={r['initial_slimes']}): {r['b03_score']} -> {r['score']} ({r['diff_b03']:+d})")

if __name__ == '__main__':
    main()
