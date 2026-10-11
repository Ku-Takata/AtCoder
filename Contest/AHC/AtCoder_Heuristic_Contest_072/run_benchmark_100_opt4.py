import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_03_opt4_chain.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_03_opt1'
b02_dir = r'outputs\B_02'
b01_dir = r'outputs\B_01'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    b02_out_file = os.path.join(b02_dir, f'{seed}.txt')

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

    # B_02 score
    b02_score = None
    if os.path.exists(b02_out_file):
        v = subprocess.run([vis_exe, in_file, b02_out_file], capture_output=True, text=True)
        for line in v.stdout.splitlines():
            if 'Score =' in line:
                b02_score = int(line.split('=')[1].strip())

    # Count initial slimes and walls
    with open(in_file) as f:
        in_lines = [l.strip() for l in f if l.strip()]
    initial_slimes = sum(c.islower() for l in in_lines[1:] for c in l)
    walls = sum(c == '#' for l in in_lines[1:] for c in l)

    return {
        'seed': seed,
        'initial_slimes': initial_slimes,
        'walls': walls,
        'score': score,
        'slimes_left': slimes_left,
        'moves': moves,
        'raw_moves': raw_moves,
        'pruned': pruned,
        'elapsed': elapsed,
        'b02_score': b02_score,
    }

def main():
    print(f'Starting 100-case benchmark for {exe} (B_03: Integrated Macro Tree Aggregation & Safe Clustering)...')
    t_start = time.time()
    results = []

    with ThreadPoolExecutor(max_workers=14) as executor:
        futures = [executor.submit(run_single, i) for i in range(100)]
        for f in futures:
            results.append(f.result())

    results.sort(key=lambda r: int(r['seed']))
    total_time = time.time() - t_start

    total_score = sum(r['score'] for r in results)
    total_b02 = sum(r['b02_score'] for r in results if r['b02_score'] is not None)
    cleared = sum(1 for r in results if r['score'] < 100000)
    
    wins = 0
    losses = 0
    ties = 0
    diff_sum = 0

    diff_list = []
    for r in results:
        if r['b02_score'] is not None:
            diff = r['score'] - r['b02_score']
            diff_sum += diff
            diff_list.append((diff, r))
            if diff < 0:
                wins += 1
            elif diff > 0:
                losses += 1
            else:
                ties += 1

    diff_list.sort(key=lambda x: x[0])
    print('\n=== Top 20 Most Improved Cases (vs B_02) ===')
    for diff, r in diff_list[:20]:
        print(f"Seed {r['seed']} (slimes={r['initial_slimes']}, walls={r['walls']}): B_02={r['b02_score']} -> B_03={r['score']} ({diff:+d})")

    print('\n=== Top 10 Most Regressed Cases (vs B_02) ===')
    for diff, r in diff_list[-10:]:
        if diff > 0:
            print(f"Seed {r['seed']} (slimes={r['initial_slimes']}, walls={r['walls']}): B_02={r['b02_score']} -> B_03={r['score']} ({diff:+d})")

    print('\n=== Tier-by-Tier Breakdown ===')
    tier_ranges = [
        ('Small (slimes <= 75)', lambda r: r['initial_slimes'] <= 75),
        ('Medium (75 < slimes <= 115)', lambda r: 75 < r['initial_slimes'] <= 115),
        ('Dense (slimes > 115)', lambda r: r['initial_slimes'] > 115),
    ]

    for label, cond in tier_ranges:
        tier_res = [r for r in results if cond(r)]
        if not tier_res: continue
        t_score = sum(r['score'] for r in tier_res)
        t_b02 = sum(r['b02_score'] for r in tier_res if r['b02_score'] is not None)
        t_diff = t_score - t_b02
        t_wins = sum(1 for r in tier_res if r['b02_score'] is not None and r['score'] < r['b02_score'])
        t_losses = sum(1 for r in tier_res if r['b02_score'] is not None and r['score'] > r['b02_score'])
        t_ties = sum(1 for r in tier_res if r['b02_score'] is not None and r['score'] == r['b02_score'])
        print(f"{label} ({len(tier_res)} cases): B_03={t_score} vs B_02={t_b02} ({t_diff:+d} moves), Win/Loss/Tie = {t_wins}/{t_losses}/{t_ties}")

    print('\n======================================================')
    print(f'Total Score (B_03): {total_score}')
    print(f'Total Score (B_02): {total_b02}')
    print(f'Net Improvement:    {diff_sum:+d} moves')
    print(f'Clear Rate:         {cleared}/100')
    print(f'vs B_02:            WIN {wins} / LOSS {losses} / TIE {ties}')
    print(f'Total Wall Time:    {total_time:.2f} s')
    print('======================================================')

if __name__ == '__main__':
    main()
