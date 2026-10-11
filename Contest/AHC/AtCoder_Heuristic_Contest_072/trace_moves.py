import os

def trace_seed(seed_int):
    seed = f'{seed_int:04d}'
    in_file = rf'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in\{seed}.txt'
    out_file = rf'outputs\B_03\{seed}.txt'

    with open(in_file) as f:
        lines = [l.strip() for l in f if l.strip()]
    n, m = map(int, lines[0].split())
    grid = [list(row) for row in lines[1:n+1]]

    # Nests
    nests = {}
    for r in range(n):
        for c in range(n):
            ch = grid[r][c]
            if 'A' <= ch <= 'Z':
                nests[ch] = (r, c)

    with open(out_file) as f:
        ops = [l.strip().split() for l in f if l.strip()]

    print(f"\n=================== TRACE SEED {seed} (N={n}, M={m}, ops={len(ops)}) ===================")
    print(f"Nests: {nests}")
    
    # 連続歩行セグメントを検出
    walk_runs = []
    cur_run = []
    for idx, op in enumerate(ops):
        r, c, k, d, l = int(op[0]), int(op[1]), int(op[2]), op[3], int(op[4])
        if k == 0:
            cur_run.append((idx, r, c, d, l))
        else:
            if len(cur_run) >= 5:
                walk_runs.append(cur_run)
            cur_run = []
    if len(cur_run) >= 5:
        walk_runs.append(cur_run)

    print(f"Long walk segments (>=5 consecutive walks): {len(walk_runs)}")
    for run in walk_runs[:5]:
        start_turn = run[0][0]
        end_turn = run[-1][0]
        length = len(run)
        first_r, first_c = run[0][1], run[0][2]
        last_r, last_c = run[-1][1], run[-1][2]
        dirs = "".join(x[3] for x in run)
        print(f"  Turn {start_turn:3d} - {end_turn:3d} ({length} walks): from ({first_r},{first_c}) to ({last_r},{last_c}) dirs=[{dirs}]")

for s in [53, 38, 75, 1, 33]:
    trace_seed(s)
