import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_01.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_01'
step44_dir = r'outputs\step44'
step63_dir = r'outputs\step63'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

os.makedirs(out_dir, exist_ok=True)

def run_single(seed_int):
    seed = f'{seed_int:04d}'
    in_file = os.path.join(in_dir, f'{seed}.txt')
    out_file = os.path.join(out_dir, f'{seed}.txt')
    step44_out_file = os.path.join(step44_dir, f'{seed}.txt')

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

    # Get Step 44 score for comparison
    score_44 = None
    if os.path.exists(step44_out_file):
        vis44_proc = subprocess.run([vis_exe, in_file, step44_out_file], capture_output=True, text=True)
        for line in vis44_proc.stdout.splitlines():
            if 'Score =' in line:
                score_44 = int(line.split('=')[1].strip())

    ac = (vis_score is not None and vis_score == score and vis_score < 100000)
    valid = (vis_score is not None and vis_score == score)

    return {
        'seed': seed,
        'seed_int': seed_int,
        'score': score,
        'vis_score': vis_score,
        'score_44': score_44,
        'slimes_left': slimes_left,
        'moves': moves,
        'raw_moves': raw_moves,
        'pruned': pruned,
        'ac': ac,
        'valid': valid,
        'elapsed': elapsed,
    }

if __name__ == '__main__':
    print('Starting 100-case parallel benchmark for B_01 (Chokudai Search + Pruning)...')
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

    cleared_results = [r for r in results if r['ac']]
    avg_moves_cleared = sum(r['moves'] for r in cleared_results) / len(cleared_results) if len(cleared_results) > 0 else 0

    print('\n========================================================')
    print(' BENCHMARK REPORT: B_01 (Chokudai Search & Operation Pruning)')
    print('========================================================')
    print(f'Total Cases Evaluated: {len(results)}')
    print(f'Valid Output:          {valid_count} / {len(results)}')
    print(f'Fully Cleared (E=0):   {ac_count} / {len(results)}')
    print(f'Total Score:           {total_score:,}')
    print(f'Average Score:         {avg_score:,.2f}')
    print(f'Avg Cleared Moves:     {avg_moves_cleared:.1f} moves (for E=0 cases)')
    print(f'Total Pruned Moves:    {total_pruned_moves:,} moves (Avg: {avg_pruned_moves:.1f} moves / case)')
    print(f'Total Elapsed Time:    {total_elapsed:.2f}s (Avg per case: {avg_elapsed:.3f}s)')
    print('========================================================')

    if any(r['score_44'] is not None for r in results):
        compared = [r for r in results if r['score_44'] is not None and r['ac']]
        wins_44 = sum(1 for r in compared if r['score'] < r['score_44'])
        losses_44 = sum(1 for r in compared if r['score'] > r['score_44'])
        ties_44 = sum(1 for r in compared if r['score'] == r['score_44'])
        print(f'Comparison vs Step 44 on Cleared Cases ({len(compared)} cases):')
        print(f'  Wins: {wins_44}, Losses: {losses_44}, Ties: {ties_44}')
        print('========================================================')
