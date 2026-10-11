import os
import subprocess

DIR_DELTA = {'U': (-1, 0), 'D': (1, 0), 'L': (0, -1), 'R': (0, 1)}

def simulate_file(seed_str):
    in_file = f'C:/Users/kuon9/Downloads/BrsBAXF2/tools/in/{seed_str}.txt'
    out_file = f'outputs/B_03/{seed_str}.txt'
    
    with open(in_file) as f:
        lines = [l.strip() for l in f if l.strip()]
    n, k = map(int, lines[0].split())
    grid = [list(l) for l in lines[1:]]

    with open(out_file) as f:
        raw_ops = [l.strip().split() for l in f if l.strip()]

    def simulate(ops):
        stacks = {}
        for r in range(n):
            for c in range(n):
                stacks[(r, c)] = []
                ch = grid[r][c]
                if ch.islower():
                    stacks[(r, c)] = [ord(ch) - ord('a')]
        
        total_slimes = sum(len(st) for st in stacks.values())
        cleared = 0
        
        for op in ops:
            r, c, k_rem, d, l = int(op[0]), int(op[1]), int(op[2]), op[3], int(op[4])
            dr, dc = DIR_DELTA[d]
            nr, nc = r + dr * l, c + dc * l
            if not (0 <= nr < n and 0 <= nc < n): return None
            if grid[nr][nc] == '#': return None
            
            st_from = stacks[(r, c)]
            if len(st_from) <= k_rem: return None
            jumping = st_from[k_rem:]
            rem = st_from[:k_rem]
            jumping.reverse()
            
            st_dest = stacks[(nr, nc)]
            if len(st_dest) + len(jumping) > 8: return None
            
            stacks[(r, c)] = rem
            stacks[(nr, nc)].extend(jumping)
            
            for p in [(r, c), (nr, nc)]:
                ch_plate = grid[p[0]][p[1]]
                if ch_plate.isupper():
                    target_col = ord(ch_plate) - ord('A')
                    while stacks[p] and stacks[p][-1] == target_col:
                        stacks[p].pop()
                        cleared += 1

        return cleared, total_slimes

    res = simulate(raw_ops)
    if not res: return 0, len(raw_ops), len(raw_ops)
    total_slimes = res[1]

    current_ops = list(raw_ops)
    improved = True
    while improved:
        improved = False
        for i in range(len(current_ops)):
            test_ops = current_ops[:i] + current_ops[i+1:]
            sim_res = simulate(test_ops)
            if sim_res and sim_res[0] == total_slimes:
                current_ops = test_ops
                improved = True
                break

    reduced = len(raw_ops) - len(current_ops)
    return reduced, len(raw_ops), len(current_ops)

total_reduced = 0
results = []
for i in range(100):
    seed_str = f"{i:04d}"
    red, orig, new_len = simulate_file(seed_str)
    if red > 0:
        results.append((seed_str, orig, new_len, red))
        total_reduced += red

print(f"Total Cases Tested: 100")
print(f"Total Moves Reduced by Pure Pruning: {total_reduced} moves!")
print(f"Number of Cases Improved: {len(results)}")
print("\nTop 15 Improvements:")
results.sort(key=lambda x: x[3], reverse=True)
for s, orig, new_len, red in results[:15]:
    print(f"Seed {s}: {orig} -> {new_len} (-{red} moves)")
