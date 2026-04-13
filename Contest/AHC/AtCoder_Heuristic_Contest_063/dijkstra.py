import sys
import heapq
import random
import time

# 実行時間計測スタート（絶対防衛線）
start_time = time.perf_counter()
input = sys.stdin.readline

N, M, C = map(int, input().split())
color = list(map(int, input().split()))
mass_flat = tuple(c for i in range(N) for c in map(int, input().split()))

snake = [(4-i) * N for i in range(5)]
colors = [1, 1, 1, 1, 1]
board = list(mass_flat)
moves = []

opposite = {'U': 'D', 'D': 'U', 'L': 'R', 'R': 'L'}
last_move = 'U'

# 乱数のシードを固定して実行ごとのブレをなくす
random.seed(42)

while True:
    # 制限時間1.8秒、または10万ターンで強制終了 (TLE / WA回避)
    if time.perf_counter() - start_time > 1.8:
        break
    if len(moves) >= 100000:
        break

    matched = 0
    limit = min(len(colors), M)
    while matched < limit and colors[matched] == color[matched]:
        matched += 1
    garbage = len(colors) - matched

    # 完璧に揃ったら終了
    if matched == M and garbage == 0:
        break

    hr, hc = snake[0] // N, snake[0] % N
    
    if garbage == 0:
        target_color = color[matched] if matched < M else color[-1]
        mode = 'bait'
        target_pos = -1
    else:
        target_color = -1
        mode = 'bite'
        target_pos = snake[matched - 2] if matched >= 2 else -1

    # 頭と尻尾の先以外を「絶対の壁」とする
    obstacles = set(snake[1:-1])
    
    dist = { (hr, hc): 0 }
    pq = [ (0, hr, hc) ]
    came_from = {}
    best_move = None
    
    # 超軽量ダイクストラ探索 (未来は考慮しない)
    while pq:
        d, r, c = heapq.heappop(pq)
        pos = r * N + c
        
        # ゴール判定
        if (mode == 'bait' and board[pos] == target_color) or \
           (mode == 'bite' and pos == target_pos):
            curr_r, curr_c = r, c
            path = []
            while (curr_r, curr_c) in came_from:
                pr, pc, d_name = came_from[(curr_r, curr_c)]
                path.append(d_name)
                curr_r, curr_c = pr, pc
            if path:
                best_move = path[-1] # 最初の1手を取得
            break
            
        for dr, dc, d_name in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
            nr, nc = r + dr, c + dc
            if 0 <= nr < N and 0 <= nc < N:
                # 最初の1手目のUターン防止
                if r == hr and c == hc and d_name == opposite[last_move]:
                    continue
                    
                npos = nr * N + nc
                cost = 1
                
                if npos in obstacles:
                    if mode == 'bite' and npos == target_pos:
                        cost = 1
                    else:
                        continue # 体には突っ込まない
                elif board[npos] > 0:
                    if mode == 'bait' and board[npos] == target_color:
                        cost = 1
                    else:
                        cost = 15 # 目標以外のゴミを食べるペナルティ
                        
                new_d = d + cost
                if new_d < dist.get((nr, nc), float('inf')):
                    dist[(nr, nc)] = new_d
                    came_from[(nr, nc)] = (r, c, d_name)
                    heapq.heappush(pq, (new_d, nr, nc))
                    
    # ★局所解からの脱出★
    if best_move is None:
        # パスが見つからない＝自分の体や壁で完全に囲まれている
        # Uターン以外のランダムな方向に1歩進んで盤面をかき混ぜる
        valid_moves = []
        bite_moves = []
        for dr, dc, d_name in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
            if d_name == opposite[last_move]:
                continue
            nr, nc = hr + dr, hc + dc
            if 0 <= nr < N and 0 <= nc < N:
                npos = nr * N + nc
                if npos not in obstacles:
                    valid_moves.append(d_name)
                else:
                    bite_moves.append(d_name)
                    
        if valid_moves:
            best_move = random.choice(valid_moves)
        elif bite_moves:
            best_move = random.choice(bite_moves) # やむを得ず体を噛みちぎる
        else:
            break # どうしようもない詰み

    moves.append(best_move)
    last_move = best_move
    
    # 状態の更新シミュレーション
    dr, dc = 0, 0
    if best_move == 'U': dr, dc = -1, 0
    elif best_move == 'D': dr, dc = 1, 0
    elif best_move == 'L': dr, dc = 0, -1
    elif best_move == 'R': dr, dc = 0, 1
    
    npos = (hr + dr) * N + hc + dc
    
    k = len(snake)
    bite_idx = -1
    for old_idx in range(2, k - 2):
        if snake[old_idx] == npos:
            bite_idx = old_idx + 1
            break
            
    if bite_idx != -1:
        h = bite_idx
        new_snake = [npos] + snake[:h]
        new_colors = colors[:h+1]
        for p in range(h + 1, k):
            board[snake[p - 1]] = colors[p]
        snake = new_snake
        colors = new_colors
    else:
        if npos == snake[-1]:
            eaten_color = 0
        else:
            eaten_color = board[npos]
            
        if eaten_color > 0:
            snake = [npos] + snake
            colors = colors + [eaten_color]
            board[npos] = 0
        else:
            snake = [npos] + snake[:-1]

for move in moves:
    print(move)