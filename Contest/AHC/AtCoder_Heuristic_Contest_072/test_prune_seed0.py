with open('C:/Users/kuon9/Downloads/BrsBAXF2/tools/in/0000.txt') as f:
    lines = [l.strip() for l in f if l.strip()]

n, k = map(int, lines[0].split())
grid = [list(l) for l in lines[1:]]

with open('outputs/B_03/0000.txt') as f:
    raw_ops = [l.strip().split() for l in f if l.strip()]

DIR_DELTA = {'U': (-1, 0), 'D': (1, 0), 'L': (0, -1), 'R': (0, 1)}

def simulate(ops):
    stacks = {}
    plates = {}
    for r in range(n):
        for c in range(n):
            stacks[(r, c)] = []
            ch = grid[r][c]
            if ch.islower():
                stacks[(r, c)] = [ord(ch) - ord('a')]
            elif ch.isupper():
                plates[ord(ch) - ord('A')] = (r, c)
    
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
        
        # Clean nests at both (r, c) and (nr, nc)
        for p in [(r, c), (nr, nc)]:
            ch_plate = grid[p[0]][p[1]]
            if ch_plate.isupper():
                target_col = ord(ch_plate) - ord('A')
                while stacks[p] and stacks[p][-1] == target_col:
                    stacks[p].pop()
                    cleared += 1

    return cleared, total_slimes

init_cleared, total = simulate(raw_ops)
print(f"Original: {len(raw_ops)} ops, cleared {init_cleared}/{total}")

# Test 1: Simple 1-op removal (Pruning)
current_ops = list(raw_ops)
improved = True
while improved:
    improved = False
    for i in range(len(current_ops)):
        test_ops = current_ops[:i] + current_ops[i+1:]
        res = simulate(test_ops)
        if res and res[0] == total:
            print(f"Pruned op at {i}: {current_ops[i]}")
            current_ops = test_ops
            improved = True
            break

print(f"After simple pruning: {len(current_ops)} ops")

# Test 2: Check Turn 3 and 4 specifically:
# Turn 2: 1 3 0 D 1 (to (2, 3))
# Turn 3: 2 3 0 R 1 (to (2, 4))
# Turn 4: 2 4 0 L 1 (to (2, 3))
# Turn 5: 2 3 0 D 1 (to (3, 3))
# If we replace Turn 3 & 4 with: (2, 4) 0 L 1 (move (2, 4) to (2, 3))
alt_ops = list(raw_ops)
# Let's inspect ops around index 3
for idx in range(2, 6):
    print(f"Op {idx}: {alt_ops[idx]}")
