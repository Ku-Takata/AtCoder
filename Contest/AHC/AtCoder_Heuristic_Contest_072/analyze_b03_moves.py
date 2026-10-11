import os

out_dir = r'outputs\B_03'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'

results = []
for i in range(100):
    seed = f'{i:04d}'
    out_file = os.path.join(out_dir, f'{seed}.txt')
    in_file = os.path.join(in_dir, f'{seed}.txt')
    with open(out_file) as f:
        ops = [line.strip().split() for line in f if line.strip()]
    
    total_ops = len(ops)
    walks = sum(1 for op in ops if int(op[2]) == 0)
    jumps = sum(1 for op in ops if int(op[2]) > 0)
    walk_ratio = walks / total_ops if total_ops > 0 else 0
    
    # 最大連続歩行
    max_consec_walk = 0
    cur_walk = 0
    consec_spans = []
    start_turn = 0
    for idx, op in enumerate(ops):
        if int(op[2]) == 0:
            if cur_walk == 0:
                start_turn = idx
            cur_walk += 1
            if cur_walk > max_consec_walk:
                max_consec_walk = cur_walk
        else:
            if cur_walk >= 6:
                consec_spans.append((start_turn, idx - 1, cur_walk))
            cur_walk = 0
    if cur_walk >= 6:
        consec_spans.append((start_turn, len(ops) - 1, cur_walk))
            
    with open(in_file) as f:
        in_lines = [l.strip() for l in f if l.strip()]
    slimes = sum(c.islower() for l in in_lines[1:] for c in l)
    walls = sum(c == '#' for l in in_lines[1:] for c in l)
    
    results.append({
        'seed': seed,
        'seed_int': i,
        'total': total_ops,
        'walks': walks,
        'jumps': jumps,
        'walk_ratio': walk_ratio,
        'max_consec_walk': max_consec_walk,
        'consec_spans': consec_spans,
        'slimes': slimes,
        'walls': walls
    })

print('=== Top 10 Max Consecutive Walks ===')
for r in sorted(results, key=lambda x: x['max_consec_walk'], reverse=True)[:10]:
    spans_str = ", ".join([f"T{s}-{e} ({l}w)" for s, e, l in r['consec_spans'][:3]])
    print(f"Seed {r['seed']}: total={r['total']}, max_consec={r['max_consec_walk']}, walks={r['walks']} ({r['walk_ratio']:.1%}), slimes={r['slimes']}, walls={r['walls']} | spans: {spans_str}")

print('\n=== Top 10 Total Moves ===')
for r in sorted(results, key=lambda x: x['total'], reverse=True)[:10]:
    print(f"Seed {r['seed']}: total={r['total']}, walks={r['walks']} ({r['walk_ratio']:.1%}), jumps={r['jumps']}, slimes={r['slimes']}, walls={r['walls']}")

print('\n=== Top 10 Walk Ratio (slimes >= 30) ===')
for r in sorted([r for r in results if r['slimes'] >= 30], key=lambda x: x['walk_ratio'], reverse=True)[:10]:
    print(f"Seed {r['seed']}: total={r['total']}, walks={r['walks']} ({r['walk_ratio']:.1%}), jumps={r['jumps']}, slimes={r['slimes']}, walls={r['walls']}")
