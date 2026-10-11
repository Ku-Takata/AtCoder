import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_02.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_02'
b01_dir = r'outputs\B_01'
appr_a_dir = r'outputs\approach_a'
appr_b_dir = r'outputs\approach_b'
appr_c_dir = r'outputs\approach_c'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b01_out_file = os.path.join(b01_dir, f'{seed}.txt')
    appr_a_out_file = os.path.join(appr_a_dir, f'{seed}.txt')

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

    # Get B_01 score
    score_b01 = None
    if os.path.exists(b01_out_file):
        vis_b01 = subprocess.run([vis_exe, in_file, b01_out_file], capture_output=True, text=True)
        for line in vis_b01.stdout.splitlines():
            if 'Score =' in line:
                score_b01 = int(line.split('=')[1].strip())

    # Get Approach A score
    score_a = None
    if os.path.exists(appr_a_out_file):
        vis_a = subprocess.run([vis_exe, in_file, appr_a_out_file], capture_output=True, text=True)
        for line in vis_a.stdout.splitlines():
            if 'Score =' in line:
                score_a = int(line.split('=')[1].strip())

    ac = (vis_score is not None and vis_score < 100000)
    valid = (vis_score is not None)

    return {
        'seed': seed,
        'seed_int': seed_int,
        'score': score,
        'vis_score': vis_score,
        'score_b01': score_b01,
        'score_a': score_a,
        'slimes_left': slimes_left,
        'moves': moves,
        'raw_moves': raw_moves,
        'pruned': pruned,
        'ac': ac,
        'valid': valid,
        'elapsed': elapsed,
    }

if __name__ == '__main__':
    print('Starting 100-case parallel benchmark for B_02 (Integrated: Tree + Catapult + Sweeper)...')
    total_start = time.time()

    with ThreadPoolExecutor(max_workers=8) as executor:
        results = list(executor.map(run_single, range(100)))

    total_elapsed = time.time() - total_start
    results.sort(key=lambda x: x['seed_int'])

    valid_count = sum(1 for r in results if r['valid'])
    ac_count = sum(1 for r in results if r['ac'])
    total_score = sum(r['score'] for r in results)
    avg_score = total_score / len(results) if len(results) > 0 else 0

    total_raw_moves = sum(r['raw_moves'] for r in results)
    total_pruned_moves = sum(r['pruned'] for r in results)
    avg_pruned_moves = total_pruned_moves / len(results) if len(results) > 0 else 0

    elapsed_list = [r['elapsed'] for r in results]
    avg_elapsed = sum(elapsed_list) / len(results) if len(results) > 0 else 0

    b01_scores = [r['score_b01'] for r in results if r['score_b01'] is not None]
    total_b01 = sum(b01_scores) if b01_scores else 0

    a_scores = [r['score_a'] for r in results if r['score_a'] is not None]
    total_a = sum(a_scores) if a_scores else 0

    better_b01 = 0
    worse_b01 = 0
    tied_b01 = 0
    diff_b01_sum = 0

    better_a = 0
    worse_a = 0
    tied_a = 0
    diff_a_sum = 0

    print(f"\n{'Seed':<6} | {'B_02':<6} | {'B_01':<6} | {'Diff(B01)':<10} | {'ApprA':<6} | {'Diff(A)':<8} | {'Time':<6}")
    print("-" * 55)
    for r in results:
        diff_b01_str = "N/A"
        if r['score_b01'] is not None:
            diff_b01 = r['score'] - r['score_b01']
            diff_b01_sum += diff_b01
            if diff_b01 < 0:
                better_b01 += 1
                diff_b01_str = f"{diff_b01:+d} (WIN)"
            elif diff_b01 > 0:
                worse_b01 += 1
                diff_b01_str = f"{diff_b01:+d} (LOSE)"
            else:
                tied_b01 += 1
                diff_b01_str = f"{diff_b01:+d}"

        diff_a_str = "N/A"
        if r['score_a'] is not None:
            diff_a = r['score'] - r['score_a']
            diff_a_sum += diff_a
            if diff_a < 0:
                better_a += 1
                diff_a_str = f"{diff_a:+d} (WIN)"
            elif diff_a > 0:
                worse_a += 1
                diff_a_str = f"{diff_a:+d} (LOSE)"
            else:
                tied_a += 1
                diff_a_str = f"{diff_a:+d}"

        print(f"{r['seed']:<6} | {r['score']:<6} | {r['score_b01'] if r['score_b01'] else 'N/A':<6} | {diff_b01_str:<10} | {r['score_a'] if r['score_a'] else 'N/A':<6} | {diff_a_str:<8} | {r['elapsed']:<5.2f}s")

    print("\n" + "=" * 60)
    print("B_02 (INTEGRATED: TREE + CATAPULT + SWEEPER) BENCHMARK SUMMARY")
    print("=" * 60)
    print(f"Total Cases:         {len(results)}")
    print(f"Valid / AC:          {valid_count} / {ac_count}")
    print(f"Total Score (Moves): {total_score}")
    print(f"  vs B_01 ({total_b01}): {diff_b01_sum:+d} moves (WIN: {better_b01}, LOSE: {worse_b01}, TIE: {tied_b01})")
    print(f"  vs ApprA ({total_a}): {diff_a_sum:+d} moves (WIN: {better_a}, LOSE: {worse_a}, TIE: {tied_a})")
    print(f"Avg Score:           {avg_score:.2f}")
    print(f"Avg Time:            {avg_elapsed:.2f}s")
    print(f"Wall Clock Time:     {total_elapsed:.2f}s")
    print("=" * 60)
