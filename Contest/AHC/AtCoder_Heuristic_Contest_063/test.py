import sys
import time
import heapq
from collections import deque

# 処理開始時間を記録
start_time = time.perf_counter()

input = sys.stdin.readline

N, M, C = map(int, input().split())
color = list(map(int, input().split()))
mass_flat = tuple(c for i in range(N) for c in map(int, input().split()))

DIR_NAMES = ['U', 'D', 'L', 'R']

# 各色の餌の初期位置をあらかじめ分類しておく（高速化の肝）
bait_positions = {i: [] for i in range(1, C + 1)}
for p, c_val in enumerate(mass_flat):
    if c_val > 0:
        bait_positions[c_val].append(p)

# 各マスから移動できる隣接マスの事前計算（定数倍高速化の要）
ADJ = [[] for _ in range(N * N)]
for p in range(N * N):
    r, c = p // N, p % N
    for dr, dc, d_str in [(-1, 0, 'U'), (1, 0, 'D'), (0, -1, 'L'), (0, 1, 'R')]:
        nr, nc = r + dr, c + dc
        if 0 <= nr < N and 0 <= nc < N:
            ADJ[p].append((nr * N + nc, d_str))

class State:
    __slots__ = ['snake', 'colors', 'board', 'moves', 'matched', 'garbage', 'score']
    
    def __init__(self, snake, colors, board, moves):
        self.snake = snake
        self.colors = colors
        self.board = board
        self.moves = moves

        m = 0
        limit = min(len(colors), M)
        while m < limit and colors[m] == color[m]:
            m += 1
        self.matched = m
        self.garbage = len(colors) - m
        
        self.score = self._calc_score()

    def _calc_score(self):
        score = self.matched * 1000000 
        score -= len(self.moves) * 10
        
        if self.matched == M and self.garbage == 0:
            return score + 10000000
            
        head_pos = self.snake[0]
        head_r, head_c = head_pos // N, head_pos % N

        next_color = color[self.matched] if self.matched < M else color[-1]
        
        min_dist = 10000
        
        # 【修正1】bait_positions を使わず、最新の盤面(self.board)から直接探す
        for p, c_val in enumerate(self.board):
            if c_val == next_color:
                r, c = p // N, p % N
                dist = abs(head_r - r) + abs(head_c - c)
                if dist < min_dist:
                    min_dist = dist
                    
        # 【修正2】盤面に目的の色がない場合は、噛みちぎれる位置にある自分の胴体（ゴミ）の中を探す
        if min_dist == 10000:
            for i, c_val in enumerate(self.colors):
                # 噛みちぎれるのはインデックス2から末尾の2つ手前まで
                if c_val == next_color and 2 <= i < len(self.snake) - 2:
                    p = self.snake[i]
                    r, c = p // N, p % N
                    dist = abs(head_r - r) + abs(head_c - c)
                    if dist < min_dist:
                        min_dist = dist
                        
        if min_dist != 10000:
            score -= min_dist * 100
            
        if self.garbage > 0:
            score -= self.garbage * 50
            if self.matched >= 2:
                target_pos = self.snake[self.matched - 2]
                tr, tc = target_pos // N, target_pos % N
                dist_to_bite = abs(head_r - tr) + abs(head_c - tc)
                score -= dist_to_bite * 100

        return score

def get_macro_paths(state, next_color):
    targets = {p for p, c in enumerate(state.board) if c == next_color}
    
    if not targets:
        for i, c in enumerate(state.colors):
            if c == next_color and 2 <= i < len(state.snake) - 2:
                targets.add(state.snake[i])
                
    if not targets:
        targets = {p for p, c in enumerate(state.board) if c > 0}
        
    # 【最重要修正】ゴミすらない完全な詰み状態は、1歩進まず即座に諦める（無限ループの防止）
    if not targets:
        return []

    head = state.snake[0]
    neck = state.snake[1] if len(state.snake) > 1 else -1 
    
    snake_set = set(state.snake)
    opposite = {'U': 'D', 'D': 'U', 'L': 'R', 'R': 'L'}
    
    required_paths = min(2, len(targets))
    
    # ---------------------------------------------------
    # 【第1段階】噛みちぎり・ゴミ禁止の超高速BFS
    # ---------------------------------------------------
    # queueの要素に「直前の方向(last_dir)」を追加
    queue = deque([(head, "", "")])
    visited_bfs = {(head, "")}
    valid_paths = []
    found_targets = set()
    
    while queue:
        curr, path, last_dir = queue.popleft()
        
        if curr in targets:
            if path and curr not in found_targets:
                valid_paths.append(path)
                found_targets.add(curr)
            if len(found_targets) >= required_paths:
                break
            continue
            
        for npos, d_str in ADJ[curr]:
            if last_dir and d_str == opposite[last_dir]:
                continue
            if not last_dir and npos == neck:
                continue
            
            # 【重要修正】訪問履歴に「進入方向」を含めて正解ルートの消滅を防ぐ
            state_key = (npos, d_str)
            if state_key not in visited_bfs:
                if state.board[npos] > 0 and npos not in targets:
                    continue
                if npos in snake_set:
                    continue
                    
                visited_bfs.add(state_key)
                queue.append((npos, path + d_str, d_str))
                    
    if valid_paths:
        return valid_paths

    # ---------------------------------------------------
    # 【第2段階】緊急回避Dijkstra
    # ---------------------------------------------------
    snake_pos_to_idx = {pos: i for i, pos in enumerate(state.snake)}
    # pqの要素に「直前の方向(last_dir)」を追加
    pq = [(0, 0, head, "", "")]
    visited_d = {(head, ""): 0}
    
    while pq:
        cost, steps, curr, path, last_dir = heapq.heappop(pq)
        
        if curr in targets:
            if path and curr not in found_targets:
                valid_paths.append(path)
                found_targets.add(curr)
            if len(found_targets) >= required_paths:
                break
            continue
            
        for npos, d_str in ADJ[curr]:
            if last_dir and d_str == opposite[last_dir]:
                continue
            if not last_dir and npos == neck:
                continue
            
            step_cost = 1
            if state.board[npos] > 0 and npos not in targets:
                step_cost += 1000
                
            if npos in snake_pos_to_idx:
                idx = snake_pos_to_idx[npos]
                if idx < len(state.snake) - steps:
                    step_cost += 50
                    
            new_cost = cost + step_cost
            
            # 【重要修正】訪問履歴に「進入方向」を含めて正解ルートの消滅を防ぐ
            state_key = (npos, d_str)
            if state_key not in visited_d or visited_d[state_key] > new_cost:
                visited_d[state_key] = new_cost
                heapq.heappush(pq, (new_cost, steps + 1, npos, path + d_str, d_str))
                    
    return valid_paths

def apply_macro_path(state, path_str):
    """ルート文字列を受け取り、途中の状態は変数だけで計算し、最後に1回だけStateを生成する"""
    curr_snake = state.snake
    curr_colors = state.colors
    curr_board = state.board
    
    for dir_name in path_str:
        hr = curr_snake[0] // N
        hc = curr_snake[0] % N
        
        nr = hr + (1 if dir_name == 'D' else -1 if dir_name == 'U' else 0)
        nc = hc + (1 if dir_name == 'R' else -1 if dir_name == 'L' else 0)
        npos = nr * N + nc
        
        k = len(curr_snake)
        bite_idx_new = -1
        for old_idx in range(2, k - 2):
            if curr_snake[old_idx] == npos:
                bite_idx_new = old_idx + 1
                break
        
        if bite_idx_new != -1:
            h = bite_idx_new
            
            # 【修正箇所】curr_snake を切り詰める「前」に、元の長さのヘビを使って盤面に色を戻す
            b_list = list(curr_board)
            for p in range(h + 1, k):
                b_list[curr_snake[p - 1]] = curr_colors[p]
                
            # 盤面を更新した「後」で、ヘビの長さと色を切り詰める
            curr_snake = (npos,) + curr_snake[:h]
            curr_colors = curr_colors[:h+1]
            curr_board = tuple(b_list)
        else:
            if npos == curr_snake[-1]:
                eaten_color = 0
            else:
                eaten_color = curr_board[npos]
                
            if eaten_color > 0:
                curr_snake = (npos,) + curr_snake
                curr_colors = curr_colors + (eaten_color,)
                b_list = list(curr_board)
                b_list[npos] = 0
                curr_board = tuple(b_list)
            else:
                curr_snake = (npos,) + curr_snake[:-1]
                
    # すべての移動が終わった最終結果だけでStateを生成する
    return State(curr_snake, curr_colors, curr_board, state.moves + path_str)

def beam_search(beam_width=30, max_turns=100000, time_limit=1.8):
    initial_snake = tuple((4-i) * N for i in range(5))
    initial_colors = tuple([1, 1, 1, 1, 1])
    initial_state = State(initial_snake, initial_colors, mass_flat, "")
    
    beam = [initial_state]
    best_state = initial_state
    
    for turn in range(max_turns):
        # ここのチェックは残しておいてOKです
        if time.perf_counter() - start_time > time_limit:
            return best_state.moves if len(best_state.moves) > 0 else make_snake_path()
            
        if not beam:
            break
            
        next_beam = []
        visited = set()
        
        for state in beam:
            # 【追加】ビーム1つ処理するごとに細かく時間をチェックし、絶対にTLEを防ぐ
            if time.perf_counter() - start_time > time_limit:
                # 途中まででも良いのでマシな解を返す。全く進んでいなければ蛇行ルート。
                return best_state.moves if len(best_state.moves) > 0 else make_snake_path()

            if state.matched == M and state.garbage == 0:
                if best_state.matched < M or best_state.garbage > 0 or len(state.moves) < len(best_state.moves):
                    best_state = state
                continue

            # --------------------------------------------------------
            # 【新規追加箇所】
            # 1歩ずつではなく、次の餌までのルートを一気に探索して状態を進める
            # --------------------------------------------------------
            next_color = color[state.matched] if state.matched < M else color[-1]
            paths = get_macro_paths(state, next_color)
            
            for path_str in paths:
                # 取得したルートで状態を一気に進める
                nxt_state = apply_macro_path(state, path_str)
                
                # 重複チェックとビームへの追加（state_keyは現在お使いのもの）
                state_key = (nxt_state.snake[0], nxt_state.snake[-1], len(nxt_state.snake), nxt_state.matched, nxt_state.garbage)
                if state_key not in visited:
                    visited.add(state_key)
                    next_beam.append(nxt_state)
        
        if not next_beam:
            break
            
        next_beam.sort(key=lambda s: s.score, reverse=True)
        beam = next_beam[:beam_width]
        
        if beam[0].score > best_state.score:
            best_state = beam[0]
            
        if best_state.matched == M and best_state.garbage == 0:
            break
            
    return best_state.moves

def make_snake_path():
    moves = []
    
    r, c = 4, 0
    direction_down = True
    
    while c < N - 1:
        if direction_down:
            while r < N - 1:
                moves.append('D')
                r += 1
        else:
            while r > 0:
                moves.append('U')
                r -= 1
        
        moves.append('R')
        c += 1
        direction_down = not direction_down

    if direction_down:
        while r < N - 1:
            moves.append('D')
            r += 1
    else:
        while r > 0:
            moves.append('U')
            r -= 1

    return "".join(moves)

# タイムリミットは1.8
ans_moves = beam_search(beam_width=8, time_limit=1.8)

if not ans_moves:
    ans_moves = "U"

for move in ans_moves:
    print(move)