import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe_base = 'B_03.exe'
exe_new = 'B_03_tier_matched.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

def run_pair(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_base = f'temp_base_{seed}.txt'
    out_new = f'temp_new_{seed}.txt'

    with open(in_file, 'r') as fin, open(out_base, 'w') as fout:
        subprocess.run([exe_base], stdin=fin, stdout=fout, stderr=subprocess.DEVNULL)
    with open(in_file, 'r') as fin, open(out_new, 'w') as fout:
        subprocess.run([exe_new], stdin=fin, stdout=fout, stderr=subprocess.DEVNULL)

    v_base = subprocess.run([vis_exe, in_file, out_base], capture_output=True, text=True)
    score_base = 0
    for l in v_base.stdout.splitlines():
        if 'Score =' in l:
            score_base = int(l.split('=')[1].strip())

    v_new = subprocess.run([vis_exe, in_file, out_new], capture_output=True, text=True)
    score_new = 0
    for l in v_new.stdout.splitlines():
        if 'Score =' in l:
            score_new = int(l.split('=')[1].strip())

    # clean up temp
    try:
        os.remove(out_base)
        os.remove(out_new)
    except:
        pass

    with open(in_file, 'r') as f:
        in_lines = [l.strip() for l in f if l.strip()]
    initial_slimes = sum(c.islower() for l in in_lines[1:] for c in l)

    return {
        'seed': seed,
        'seed_int': seed_int,
        'initial_slimes': initial_slimes,
        'score_base': score_base,
        'score_new': score_new,
        'diff': score_new - score_base,
    }

def main():
    print("Running true head-to-head evaluation (B_03.exe vs B_03_tier_matched.exe)...")
    t0 = time.time()
    results = []
    with ThreadPoolExecutor(max_workers=8) as executor:
        futures = [executor.submit(run_pair, i) for i in range(100)]
        for f in futures:
            results.append(f.result())

    results.sort(key=lambda x: x['seed_int'])
    t_total = time.time() - t0

    tot_base = sum(r['score_base'] for r in results)
    tot_new = sum(r['score_new'] for r in results)
    tot_diff = tot_new - tot_base

    small = [r for r in results if r['initial_slimes'] <= 75]
    med = [r for r in results if 75 < r['initial_slimes'] <= 115]
    dense = [r for r in results if r['initial_slimes'] > 115]

    improved = [r for r in results if r['diff'] < 0]
    degraded = [r for r in results if r['diff'] > 0]
    tied = [r for r in results if r['diff'] == 0]

    print("\n" + "="*60)
    print("TRUE HEAD-TO-HEAD BENCHMARK SUMMARY")
    print("="*60)
    print(f"Total Base: {tot_base:,} moves | Total New: {tot_new:,} moves")
    print(f"Net Difference: {tot_diff:+d} moves ({'IMPROVEMENT!' if tot_diff < 0 else 'DEGRADATION'})")
    print(f"Win/Tie/Loss  : {len(improved)} wins / {len(tied)} ties / {len(degraded)} losses")
    print(f"Total Time    : {t_total:.1f}s")
    print("-" * 60)
    print(f"Small Tier  (<=75) : {sum(r['score_new'] for r in small):,} vs {sum(r['score_base'] for r in small):,} ({sum(r['diff'] for r in small):+d}) [n={len(small)}]")
    print(f"Medium Tier (76-115): {sum(r['score_new'] for r in med):,} vs {sum(r['score_base'] for r in med):,} ({sum(r['diff'] for r in med):+d}) [n={len(med)}]")
    print(f"Dense Tier  (>115) : {sum(r['score_new'] for r in dense):,} vs {sum(r['score_base'] for r in dense):,} ({sum(r['diff'] for r in dense):+d}) [n={len(dense)}]")
    print("-" * 60)

if __name__ == '__main__':
    main()
