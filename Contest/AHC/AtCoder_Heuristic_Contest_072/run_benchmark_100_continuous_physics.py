import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_04_continuous_physics.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\continuous_physics'
b04_dir = r'outputs\B_04'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b04_out_file = os.path.join(b04_dir, f'{seed}.txt')

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

    # Read b04 score
    b04_score = None
    if os.path.exists(b04_out_file):
        v = subprocess.run([vis_exe, in_file, b04_out_file], capture_output=True, text=True)
        for line in v.stdout.splitlines():
            if 'Score =' in line:
                b04_score = int(line.split('=')[1].strip())

    # Count initial slimes and density
    with open(in_file, 'r') as f:
        in_lines = [l.strip() for l in f if l.strip()]
    n = int(in_lines[0].split()[0])
    initial_slimes = sum(c.islower() for l in in_lines[1:] for c in l)
    density = initial_slimes / (n * n)

    return {
        'seed': seed,
        'seed_int': seed_int,
        'n': n,
        'initial_slimes': initial_slimes,
        'density': density,
        'score': score,
        'b04_score': b04_score,
        'diff_b04': score - b04_score if b04_score is not None else 0,
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
    total_b04 = sum(r['b04_score'] for r in results if r['b04_score'] is not None)
    total_diff = total_score - total_b04
    unreturned_cases = [r['seed'] for r in results if r['score'] >= 100000]
    improved = [r for r in results if r['diff_b04'] < 0]
    degraded = [r for r in results if r['diff_b04'] > 0]
    tied = [r for r in results if r['diff_b04'] == 0]

    # Analysis across continuous density spectrum
    sparse = [r for r in results if r['density'] < 0.35]
    medium = [r for r in results if 0.35 <= r['density'] < 0.45]
    dense = [r for r in results if r['density'] >= 0.45]

    print("\n" + "="*60)
    print("BENCHMARK SUMMARY (Continuous Physics vs B_04 baseline 33,098)")
    print("="*60)
    print(f"Total Score (moves): {total_score:,} (B_04 baseline: {total_b04:,})")
    print(f"Overall Difference : {total_diff:+d} moves ({'IMPROVEMENT!' if total_diff < 0 else 'DEGRADATION'})")
    print(f"Total time         : {t_total:.1f}s")
    print(f"Unreturned cases   : {len(unreturned_cases)}")
    print(f"Win/Tie/Loss       : {len(improved)} wins / {len(tied)} ties / {len(degraded)} losses")
    print("-" * 60)
    print("Density spectrum breakdown:")
    print(f"Sparse (density < 0.35)  : {sum(r['score'] for r in sparse):,} vs {sum(r['b04_score'] for r in sparse):,} ({sum(r['score'] for r in sparse)-sum(r['b04_score'] for r in sparse):+d}) [n={len(sparse)}]")
    print(f"Medium (0.35 <= rho < 0.45): {sum(r['score'] for r in medium):,} vs {sum(r['b04_score'] for r in medium):,} ({sum(r['score'] for r in medium)-sum(r['b04_score'] for r in medium):+d}) [n={len(medium)}]")
    print(f"Dense  (rho >= 0.45)     : {sum(r['score'] for r in dense):,} vs {sum(r['b04_score'] for r in dense):,} ({sum(r['score'] for r in dense)-sum(r['b04_score'] for r in dense):+d}) [n={len(dense)}]")
    print("-" * 60)

    if improved:
        print("\nTop 5 Improvements:")
        for r in sorted(improved, key=lambda x: x['diff_b04'])[:5]:
            print(f"  Seed {r['seed']} (N={r['n']}, slimes={r['initial_slimes']}, rho={r['density']:.3f}): {r['b04_score']} -> {r['score']} ({r['diff_b04']:+d})")
    if degraded:
        print("\nTop 5 Degradations:")
        for r in sorted(degraded, key=lambda x: -x['diff_b04'])[:5]:
            print(f"  Seed {r['seed']} (N={r['n']}, slimes={r['initial_slimes']}, rho={r['density']:.3f}): {r['b04_score']} -> {r['score']} ({r['diff_b04']:+d})")

    # Also calculate the combined best (portfolio)
    portfolio_best = sum(min(r['score'], r['b04_score']) for r in results)
    print("\n" + "="*60)
    print(f"Portfolio Best (Min(B_04, Continuous)): {portfolio_best:,} (diff from B_04: {portfolio_best - total_b04:+d})")
    print("="*60)

if __name__ == '__main__':
    main()
