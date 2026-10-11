with open('C:/Users/kuon9/Downloads/BrsBAXF2/tools/in/0000.txt') as f:
    lines = [l.strip() for l in f if l.strip()]

n, k = map(int, lines[0].split())
grid = [list(l) for l in lines[1:]]

# board stacks: dict from (r, c) -> list of color char
stacks = {}
plates = {}
for r in range(n):
    for c in range(n):
        stacks[(r, c)] = []
        ch = grid[r][c]
        if ch.islower():
            stacks[(r, c)] = [ch]
        elif ch.isupper():
            plates[ch] = (r, c)

with open('outputs/B_03/0000.txt') as f:
    ops = [l.strip().split() for l in f if l.strip()]

DIR_DELTA = {'U': (-1, 0), 'D': (1, 0), 'L': (0, -1), 'R': (0, 1)}

print("=== Initial around (2, 3), (2, 4) ===")
for r in range(n):
    for c in range(n):
        if stacks[(r, c)]:
            print(f"({r}, {c}): {stacks[(r, c)]}")

for t, op in enumerate(ops[:8]):
    r, c, k_rem, d, l = int(op[0]), int(op[1]), int(op[2]), op[3], int(op[4])
    dr, dc = DIR_DELTA[d]
    nr, nc = r + dr * l, c + dc * l
    
    st_from = stacks[(r, c)]
    jumping = st_from[k_rem:]
    rem = st_from[:k_rem]
    jumping.reverse()
    
    stacks[(r, c)] = rem
    stacks[(nr, nc)].extend(jumping)
    
    print(f"\n--- After Turn {t}: {r} {c} {k_rem} {d} {l} (to ({nr}, {nc})) ---")
    print(f"From ({r}, {c}): remaining {stacks[(r, c)]}")
    print(f"Dest ({nr}, {nc}): now {stacks[(nr, nc)]}")
    # Also check nests
    # (assuming nest check for simplicity)
