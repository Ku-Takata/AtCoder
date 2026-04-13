import sys
import time
import heapq

# 処理開始時間を記録
start_time = time.perf_counter()
input = sys.stdin.readline

N, M, C = map(int, input().split())
color = list(map(int, input().split()))
mass_flat = list(c for i in range(N) for c in map(int, input().split()))

DIR_NAMES = ['U', 'D', 'L', 'R']
MAX_TURNS = 100000

bait_positions = {i: [] for i in range(1, C + 1)}
for p, c_val in enumerate(mass_flat):
    if c_val > 0:
        bait_positions[c_val].append(p)

board = list(mass_flat)
snake = [(4-i) * N for i in range(5)]
colors = [1, 1, 1, 1, 1]

ans_moves = ""
last_move = ""

def update_matched():
    """現在のヘビの配列が、目的の配列とどこまで一致しているかを正確に計算する"""
    m = 0
    limit = min(len(colors) - 5, M)
    while m < limit and colors[5 + m] == color[m]:
        m += 1
    return m

def get_safe_move(prev_move):
    """どうしようもない時の安全な1手。行ったり来たりを防ぎ、尻尾を追う"""
    # 1. まずは自分の尻尾を目指して空間をこじ開ける
    tail_path = find_bfs_path(snake[0], [snake[-1]], avoid_colors=False)
    if tail_path:
        return tail_path[0]
        
    # 2. 尻尾への道もない場合は、直前の逆方向を避けて1歩動く
    hr, hc = snake[0] // N, snake[0] % N
    pr, pc = snake[1] // N, snake[1] % N
    
    opposite = {'U':'D', 'D':'U', 'L':'R', 'R':'L'}
    rev_move = opposite.get(prev_move, '')
    
    safe_empty = []
    safe_eat = []
    
    for d in DIR_NAMES:
        nr = hr + (1 if d == 'D' else -1 if d == 'U' else 0)
        nc = hc + (1 if d == 'R' else -1 if d == 'L' else 0)
        if 0 <= nr < N and 0 <= nc < N:
            if nr != pr or nc != pc:
                npos = nr * N + nc
                if npos not in snake[:-1]: # 体にぶつからない（尻尾はOK）
                    if board[npos] == 0:
                        safe_empty.append(d)
                    else:
                        safe_eat.append(d)
                        
    # 反復横跳び（行ったり来たり）をなるべく避けるため、逆方向の優先度を下げる
    for m_list in (safe_empty, safe_eat):
        if len(m_list) > 1 and rev_move in m_list:
            m_list.remove(rev_move)
            m_list.append(rev_move)
            
    if safe_empty: return safe_empty[0]
    if safe_eat: return safe_eat[0]
    return 'U'

def execute_move(move_char):
    """1手進め、状態（食事・噛みちぎり）が変わった場合はTrueを返す"""
    global snake, colors, board, ans_moves
    if len(ans_moves) >= MAX_TURNS:
        return False
        
    hr, hc = snake[0] // N, snake[0] % N
    nr = hr + (1 if move_char == 'D' else -1 if move_char == 'U' else 0)
    nc = hc + (1 if move_char == 'R' else -1 if move_char == 'L' else 0)
    
    # 盤外に出てしまう場合は無視（エラー落ち防止）
    if not (0 <= nr < N and 0 <= nc < N):
        ans_moves += move_char 
        return True
        
    npos = nr * N + nc
    ans_moves += move_char
    
    k = len(snake)
    bite_idx = -1
    for h in range(1, k - 1):
        if npos == snake[h - 1]:
            bite_idx = h
            break
            
    if bite_idx != -1:
        # 噛みちぎり発生
        h = bite_idx
        new_snake = [npos] + snake[:h]
        new_colors = colors[:h+1]
        
        # 噛みちぎった部分を盤面に落とす
        for p_idx in range(h + 1, k):
            pos = snake[p_idx - 1]
            c = colors[p_idx]
            board[pos] = c
            if c > 0: bait_positions[c].append(pos)
            
        snake = new_snake
        colors = new_colors
        return True
        
    eaten = 0 if npos == snake[-1] else board[npos]
    if eaten > 0:
        # 食事発生
        snake.insert(0, npos)
        colors.append(eaten)
        board[npos] = 0
        return True
    else:
        # 通常移動
        snake.insert(0, npos)
        snake.pop()
        return False

def find_bfs_path(start_pos, target_positions, avoid_colors=False):
    """BFSでの経路探索"""
    q = [(0, start_pos, "")]
    visited = {start_pos}
    snake_body = set(snake)
    
    while q:
        cost, pos, path = heapq.heappop(q)
        if pos in target_positions:
            return path
            
        r, c = pos // N, pos % N
        for dir_name in DIR_NAMES:
            nr = r + (1 if dir_name == 'D' else -1 if dir_name == 'U' else 0)
            nc = c + (1 if dir_name == 'R' else -1 if dir_name == 'L' else 0)
            
            if 0 <= nr < N and 0 <= nc < N:
                npos = nr * N + nc
                if path == "" and npos == snake[1]: continue # Uターン防止
                
                is_target = (npos in target_positions)
                is_tail = (npos == snake[-1])
                
                # ターゲット自体、または尻尾（逃げるので踏んでOK）なら進める
                if npos not in visited and (npos not in snake_body or is_target or is_tail):
                    if avoid_colors and board[npos] > 0 and not is_target:
                        continue
                    visited.add(npos)
                    # 壁沿いボーナスは少し弱めにし、純粋な距離を優先
                    wall_dist = min(nr, N-1-nr, nc, N-1-nc)
                    heapq.heappush(q, (cost + 1 + wall_dist, npos, path + dir_name))
    return None


# ================= メインループ =================
while len(ans_moves) < MAX_TURNS:
    if time.perf_counter() - start_time > 1.8:
        break
        
    matched = update_matched()
    if matched >= M:
        break # 全て揃った！
        
    expected_len = 5 + matched
    has_garbage = len(snake) > expected_len
    
    path = None
    
    if has_garbage:
        # ゴミを持っていたら、ただちに「本来あるべき長さの尻尾」に頭をぶつけて噛みちぎる
        target_bite_pos = snake[expected_len - 2]
        path = find_bfs_path(snake[0], [target_bite_pos], avoid_colors=False)
    else:
        # ターゲット探索モード
        target_c = color[matched]
        targets = [p for p in bait_positions[target_c] if board[p] == target_c]
        
        if targets:
            path = find_bfs_path(snake[0], targets, avoid_colors=True)
            if not path:
                path = find_bfs_path(snake[0], targets, avoid_colors=False)
                
    # 経路が見つからなければ、安全な1歩をひねり出して体をほぐす
    if not path:
        path = get_safe_move(last_move)
        
    # 経路を1歩ずつ実行する
    for move_char in path:
        state_changed = execute_move(move_char)
        last_move = move_char
        if state_changed:
            break

if not ans_moves:
    ans_moves = "U"

for move in ans_moves:
    print(move)