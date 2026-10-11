import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_06_idea3_dense_lns.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_06_idea3'
b06_dir = r'outputs\B_06'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b06_out_file = os.path.join(b06_dir, f'{seed}.txt')

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

    # Read b06 score
    b06_score = None
    if os.path.exists(b06_out_file):
        vis_b06 = subprocess.run([vis_exe, in_file, b06_out_file], capture_output=True, text=True)
        for line in vis_b06.stdout.splitlines():
            if 'Score =' in line:
                b06_score = int(line.split('=')[1].strip())

    # Parse initial slimes
    initial_slimes = 0
    with open(in_file, 'r') as fin:
        for line in fin.readlines()[1:]:
            for ch in line:
                if ch.islower():
                    initial_slimes += 1

    diff = score - b06_score if b06_score is not None else 0
    return {
        'seed': seed,
        'score': score,
        'b06_score': b06_score,
        'diff_b06': diff,
        'elapsed': elapsed,
        'initial_slimes': initial_slimes,
    }

def main():
    t_start = time.time()
    results = []

    with ThreadPoolExecutor(max_workers=4) as executor:
        futures = [executor.submit(run_single, i) for i in range(100)]
        for f in futures:
            res = f.result()
            results.append(res)
            d_str = f"{res['diff_b06']:+d}" if res['diff_b06'] != 0 else "="
            print(f"[{res['seed']}] score={res['score']} (B_06={res['b06_score']}, diff={d_str}) time={res['elapsed']:.2f}s slimes={res['initial_slimes']}")

    t_total = time.time() - t_start
    results.sort(key=lambda x: int(x['seed']))

    total_score = sum(r['score'] for r in results)
    total_b06 = sum(r['b06_score'] for r in results if r['b06_score'] is not None)
    total_diff = total_score - total_b06

    unreturned_cases = [r['seed'] for r in results if r['score'] >= 100000]
    improved = [r for r in results if r['diff_b06'] < 0]
    degraded = [r for r in results if r['diff_b06'] > 0]
    tied = [r for r in results if r['diff_b06'] == 0]

    small = [r for r in results if r['initial_slimes'] <= 75]
    med = [r for r in results if 75 < r['initial_slimes'] <= 115]
    dense = [r for r in results if r['initial_slimes'] > 115]

    print("\n" + "="*60)
    print("BENCHMARK SUMMARY (B_06_idea3 vs B_06 baseline 32,193)")
    print("="*60)
    print(f"Total Score (moves): {total_score:,} (B_06 baseline: {total_b06:,})")
    print(f"Overall Difference : {total_diff:+d} moves ({'IMPROVEMENT!' if total_diff < 0 else 'DEGRADATION'})")
    print(f"Total time         : {t_total:.1f}s")
    print(f"Unreturned cases   : {len(unreturned_cases)}")
    print(f"Win/Tie/Loss       : {len(improved)} wins / {len(tied)} ties / {len(degraded)} losses")
    print("-" * 60)
    print(f"Small Tier  (<=75) : {sum(r['score'] for r in small):,} vs {sum(r['b06_score'] for r in small):,} ({sum(r['score'] for r in small)-sum(r['b06_score'] for r in small):+d}) [n={len(small)}]")
    print(f"Medium Tier (76-115): {sum(r['score'] for r in med):,} vs {sum(r['b06_score'] for r in med):,} ({sum(r['score'] for r in med)-sum(r['b06_score'] for r in med):+d}) [n={len(med)}]")
    print(f"Dense Tier  (>115) : {sum(r['score'] for r in dense):,} vs {sum(r['b06_score'] for r in dense):,} ({sum(r['score'] for r in dense)-sum(r['b06_score'] for r in dense):+d}) [n={len(dense)}]")
    print("-" * 60)

if __name__ == '__main__':
    main()
