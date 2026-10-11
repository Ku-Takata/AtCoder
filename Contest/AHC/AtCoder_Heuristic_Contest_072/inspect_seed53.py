import os

seed = '0053'
in_file = rf'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in\{seed}.txt'
out_file = rf'outputs\B_03\{seed}.txt'

with open(in_file) as f:
    lines = [l.strip() for l in f if l.strip()]
n, m = map(int, lines[0].split())
grid = [list(row) for row in lines[1:n+1]]

# Initial board
# board[r][c] = list of chars (bottom to top)
board = [[[] for _ in range(n)] for _ in range(n)]
for r in range(n):
    for c in range(n):
        ch = grid[r][c]
        if 'a' <= ch <= 'z':
            board[r][c].append(ch)

with open(out_file) as f:
    ops = [l.strip().split() for l in f if l.strip()]

dirs_delta = {'U': (-1, 0), 'D': (1, 0), 'L': (0, -1), 'R': (0, 1)}

print(f"Tracking Seed {seed} around Turn 61 - 115:")
for idx, op in enumerate(ops):
    r, c, k, d, l = int(op[0]), int(op[1]), int(op[2]), op[3], int(op[4])
    dr, dc = dirs_delta[d]
    
    # State before move
    slimes_at_src = board[r][c]
    h_src = len(slimes_at_src)
    
    if 60 <= idx <= 112:
        top_color = slimes_at_src[-1] if h_src > 0 else '?'
        print(f"Turn {idx:3d}: pos=({r:2d},{c:2d}) h={h_src} top={top_color} op={k} {d} {l}", end="")
        
    # Apply op
    # Pop top l slimes from (r, c)
    moving = slimes_at_src[h_src - l:]
    board[r][c] = slimes_at_src[:h_src - l]
    # Reverse when moving
    moving.reverse()
    
    # Destination
    nr = r + dr * (k + 1)
    nc = c + dc * (k + 1)
    
    if 60 <= idx <= 112:
        dest_h_before = len(board[nr][nc])
        dest_top = board[nr][nc][-1] if dest_h_before > 0 else 'empty'
        print(f" -> to ({nr:2d},{nc:2d}) dest_h={dest_h_before}({dest_top})")
        
    board[nr][nc].extend(moving)
