import subprocess
import os
import time
from concurrent.futures import ThreadPoolExecutor

exe = 'B_06_idea4_sa_feedback.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_06_idea4'
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
    seeds = list(range(100))
    results = []

    # Run in parallel with max_workers=6
    with ThreadPoolExecutor(max_workers=6) as pool:
        for res in pool.map(run_single, seeds):
            results.append(res)
            # progress indication
            if len(results) % 20 == 0:
                print(f"Done {len(results)}/100 tests...")

    results.sort(key=lambda x: x['seed'])

    total_score = sum(r['score'] for r in results)
    total_b06 = sum(r['b06_score'] for r in results if r['b06_score'] is not None)
    total_diff = total_score - total_b06

    wins = sum(1 for r in results if r['diff_b06'] < 0)
    ties = sum(1 for r in results if r['diff_b06'] == 0)
    losses = sum(1 for r in results if r['diff_b06'] > 0)
    zero_scores = sum(1 for r in results if r['score'] == 0)

    print("\n" + "="*50)
    print(f"BENCHMARK RESULTS: B_06_idea4 (SA Real Ops Feedback) vs B_06 Baseline")
    print("="*50)
    print(f"Total Score (Idea 4) : {total_score} ops")
    print(f"Total Score (B_06)   : {total_b06} ops")
    print(f"Score Difference     : {total_diff:+d} ops ({total_diff/total_b06*100:+.2f}%)")
    print(f"Win / Tie / Loss     : {wins} W / {ties} T / {losses} L")
    print(f"Invalid (Score=0)    : {zero_scores}")
    print(f"Total Elapsed Time   : {time.time() - t_start:.2f} s")
    print("="*50)

    # Detailed tier breakdown
    print("\nBreakdown by Slime Density Tier:")
    tiers = [
        ("Small (K <= 10)", lambda r: r['initial_slimes'] <= 10),
        ("Medium (11 <= K <= 20)", lambda r: 11 <= r['initial_slimes'] <= 20),
        ("Dense (K >= 21)", lambda r: r['initial_slimes'] >= 21),
    ]

    for name, pred in tiers:
        sub = [r for r in results if pred(r)]
        if not sub:
            continue
        sub_score = sum(r['score'] for r in sub)
        sub_b06 = sum(r['b06_score'] for r in sub if r['b06_score'] is not None)
        sub_diff = sub_score - sub_b06
        sub_w = sum(1 for r in sub if r['diff_b06'] < 0)
        sub_t = sum(1 for r in sub if r['diff_b06'] == 0)
        sub_l = sum(1 for r in sub if r['diff_b06'] > 0)
        print(f"  {name:22s}: {sub_score} vs {sub_b06} ({sub_diff:+d}) | {sub_w}W {sub_t}T {sub_l}L (n={len(sub)})")

if __name__ == '__main__':
    main()
