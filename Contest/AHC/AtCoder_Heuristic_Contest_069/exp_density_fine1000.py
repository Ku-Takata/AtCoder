import concurrent.futures
import math
import os
import re
import subprocess
import time

EXECUTABLE = './main6.exe'
TESTER = r'AdcJXWH4_windows\\tools_x86_64-pc-windows-gnu\\tester.exe'
IN_DIR = r'AdcJXWH4_windows\\tools_x86_64-pc-windows-gnu\\in'

fn_score = re.compile(r'Score\s*=\s*(\d+)')
fn_p1_info = re.compile(r'Phase 1 \(C > 0\.95\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)')
fn_p2_info = re.compile(r'Phase 2 \(C > 0\.85\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)')
fn_p25_info = re.compile(r'Phase 2\.5 \(C > 0\.75\)\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)')
fn_p3_info = re.compile(r'Phase 3 / MultiMove\s*:\s*count=\s*(\d+)\s*\|\s*fee=\s*(\d+)')
fn_moves = re.compile(r'Moves:\s*(\d+)\s*\(Cost:\s*(\d+)\)')

def run_single_case(args):
    seed, mult = args
    seed_str = f'{seed:04d}'
    in_file = os.path.join(IN_DIR, f'{seed_str}.txt')
    if not os.path.exists(in_file):
        return None

    dummy_params = [
        '0.000930648811439626', '0.0015812296573779147', '0.004643700270792169', '0.0008822059737903149',
        '0.0019684143784392803', '0.0010232407503122955', '0.003947946318465455', '0.0006210630252290683',
        '1.8232673356521643', '1.5458212320758866', '1.4074403758571954', '514.1966045939967',
        '0.85', '300000.0', '0.90', '1.5', '2.0', '1000000.0', '3', '4',
        str(mult)
    ]
    cmd = [TESTER, EXECUTABLE] + dummy_params
    
    t0 = time.time()
    with open(in_file, 'r', encoding='utf-8') as fin:
        res = subprocess.run(cmd, stdin=fin, capture_output=True, text=True, encoding='utf-8')

    stderr = res.stderr or ''
    stdout = res.stdout or ''
    combined = stdout + '\n' + stderr

    score = int(fn_score.search(combined).group(1)) if fn_score.search(combined) else 0
    p1 = int(fn_p1_info.search(stderr).group(1)) if fn_p1_info.search(stderr) else 0
    p2 = int(fn_p2_info.search(stderr).group(1)) if fn_p2_info.search(stderr) else 0
    p25 = int(fn_p25_info.search(stderr).group(1)) if fn_p25_info.search(stderr) else 0
    p3 = int(fn_p3_info.search(stderr).group(1)) if fn_p3_info.search(stderr) else 0
    accepts = p1 + p2 + p25 + p3
    
    fee1 = int(fn_p1_info.search(stderr).group(2)) if fn_p1_info.search(stderr) else 0
    fee2 = int(fn_p2_info.search(stderr).group(2)) if fn_p2_info.search(stderr) else 0
    fee25 = int(fn_p25_info.search(stderr).group(2)) if fn_p25_info.search(stderr) else 0
    fee3 = int(fn_p3_info.search(stderr).group(2)) if fn_p3_info.search(stderr) else 0
    total_fee = fee1 + fee2 + fee25 + fee3

    moves = int(fn_moves.search(stderr).group(1)) if fn_moves.search(stderr) else 0
    move_cost = int(fn_moves.search(stderr).group(2)) if fn_moves.search(stderr) else 0

    return {
        'seed': seed, 'score': score, 'fee': total_fee,
        'accepts': accepts, 'moves': moves, 'move_cost': move_cost
    }

def run_experiment(mult, num_seeds=1000):
    tasks = [(s, mult) for s in range(num_seeds)]
    results = []
    t0 = time.time()
    with concurrent.futures.ThreadPoolExecutor(max_workers=12) as executor:
        for r in executor.map(run_single_case, tasks):
            if r:
                results.append(r)
    elapsed = time.time() - t0
    
    avg_score = sum(r['score'] for r in results) / len(results)
    avg_fee = sum(r['fee'] for r in results) / len(results)
    avg_accepts = sum(r['accepts'] for r in results) / len(results)
    avg_moves = sum(r['moves'] for r in results) / len(results)
    avg_move_cost = sum(r['move_cost'] for r in results) / len(results)
    
    print(f'mult={mult:.2f} | Time: {elapsed:5.1f}s | Avg Score: {avg_score:10.1f} | Avg Fee: {avg_fee:10.1f} | Avg Accepts: {avg_accepts:5.1f} | Avg Move Cost: {avg_move_cost:8.1f}', flush=True)
    return {
        'mult': mult, 'avg_score': avg_score, 'avg_fee': avg_fee,
        'avg_accepts': avg_accepts, 'avg_moves': avg_moves, 'avg_move_cost': avg_move_cost
    }

if __name__ == '__main__':
    # 1.00, 1.01, 1.02, 1.03, 1.04, 1.05, 1.06
    mults = [1.00, 1.01, 1.02, 1.03, 1.04, 1.05, 1.06]
    for m in mults:
        run_experiment(m, num_seeds=1000)
