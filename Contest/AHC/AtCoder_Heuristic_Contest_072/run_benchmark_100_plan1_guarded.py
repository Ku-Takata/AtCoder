import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_03_plan1_guarded.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_03_plan1_guarded'
b02_dir = r'outputs\B_02'
b03_dir = r'outputs\B_03'
b01_dir = r'outputs\B_01'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b02_out_file = os.path.join(b02_dir, f'{seed}.txt')
    b03_out_file = os.path.join(b03_dir, f'{seed}.txt')

    t0 = time.time()
    with open(in_file, 'r') as fin, open(out_file, 'w') as fout:
        proc = subprocess.run([exe], stdin=fin, stdout=fout, stderr=subprocess.PIPE, text=True)
    t1 = time.time()
    elapsed = t1 - t0

    score = 0
    slimes_left = 999
    moves = 0
    raw_moves = 0
    pruned = 0
    for line in proc.stderr.splitlines():
        if 'STAT:' in line:
            parts = line.split()
            for p in parts:
                if p.startswith('best_score='):
                    score = int(p.split('=')[1])
                elif p.startswith('final_score='):
                    score = int(p.split('=')[1])
                elif p.startswith('slimes_left='):
                    slimes_left = int(p.split('=')[1])
                elif p.startswith('moves='):
                    moves = int(p.split('=')[1])
                elif p.startswith('raw_moves='):
                    raw_moves = int(p.split('=')[1])
                elif p.startswith('pruned='):
                    pruned = int(p.split('=')[1])

    # Validate with vis.exe
    vis_proc = subprocess.run([vis_exe, in_file, out_file], capture_output=True, text=True)
    vis_score = None
    for line in vis_proc.stdout.splitlines():
        if 'Score =' in line:
            vis_score = int(line.split('=')[1].strip())

    if vis_score is not None:
        score = vis_score

    # Read b03 score
    b03_score = None
    if os.path.exists(b03_out_file):
        v = subprocess.run([vis_exe, in_file, b03_out_file], capture_output=True, text=True)
        for line in v.stdout.splitlines():
            if 'Score =' in line:
                b03_score = int(line.split('=')[1].strip())

    return {
        'seed': seed,
        'seed_int': seed_int,
        'score': score,
        'b03_score': b03_score,
        'diff_b03': score - b03_score if b03_score is not None else 0,
        'slimes_left': slimes_left,
        'moves': moves,
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
    max_slimes_left = max(r['slimes_left'] for r in results)
    unreturned_cases = [r['seed'] for r in results if r['slimes_left'] > 0]
    improved = [r for r in results if r['diff_b03'] < 0]
    degraded = [r for r in results if r['diff_b03'] > 0]
    tied = [r for r in results if r['diff_b03'] == 0]

    # Tier statistics
    small_scores = [r['score'] for r in results if r['seed_int'] < 100 and read_total_slimes(r['seed_int']) <= 75]
    small_b03 = [r['b03_score'] for r in results if r['seed_int'] < 100 and read_total_slimes(r['seed_int']) <= 75]
    
    med_scores = [r['score'] for r in results if 75 < read_total_slimes(r['seed_int']) <= 115]
    med_b03 = [r['b03_score'] for r in results if 75 < read_total_slimes(r['seed_int']) <= 115]

    dense_scores = [r['score'] for r in results if read_total_slimes(r['seed_int']) > 115]
    dense_b03 = [r['b03_score'] for r in results if read_total_slimes(r['seed_int']) > 115]

    print("\n" + "="*60)
    print("BENCHMARK SUMMARY (B_03_plan1_guarded vs B_03 baseline)")
    print("="*60)
    print(f"Total Score (moves): {total_score:,} (B_03 baseline: {total_b03:,})")
    print(f"Overall Difference : {total_diff:+d} moves ({'IMPROVEMENT!' if total_diff < 0 else 'DEGRADATION'})")
    print(f"Total time         : {t_total:.1f}s")
    print(f"Unreturned cases   : {len(unreturned_cases)} (max slimes left: {max_slimes_left})")
    print(f"Win/Tie/Loss       : {len(improved)} wins / {len(tied)} ties / {len(degraded)} losses")
    print("-" * 60)
    print(f"Small Tier  (<=75) : {sum(small_scores):,} vs {sum(small_b03):,} ({sum(small_scores)-sum(small_b03):+d}) [n={len(small_scores)}]")
    print(f"Medium Tier (76-115): {sum(med_scores):,} vs {sum(med_b03):,} ({sum(med_scores)-sum(med_b03):+d}) [n={len(med_scores)}]")
    print(f"Dense Tier  (>115) : {sum(dense_scores):,} vs {sum(dense_b03):,} ({sum(dense_scores)-sum(dense_b03):+d}) [n={len(dense_scores)}]")
    print("-" * 60)

    if improved:
        print("\nTop 5 Improvements:")
        for r in sorted(improved, key=lambda x: x['diff_b03'])[:5]:
            print(f"  Seed {r['seed']}: {r['b03_score']} -> {r['score']} ({r['diff_b03']:+d})")
    if degraded:
        print("\nTop 5 Degradations:")
        for r in sorted(degraded, key=lambda x: x['diff_b03'], reverse=True)[:5]:
            print(f"  Seed {r['seed']}: {r['b03_score']} -> {r['score']} ({r['diff_b03']:+d})")

def read_total_slimes(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    with open(in_file, 'r') as f:
        in_lines = [l.strip() for l in f if l.strip()]
    return sum(c.islower() for l in in_lines[1:] for c in l)

if __name__ == '__main__':
    main()

