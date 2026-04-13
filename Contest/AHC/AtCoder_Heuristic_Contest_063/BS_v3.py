import sys
import time

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
        # 盤面全体ではなく、その色が存在する座標リストだけをループする
        for p in bait_positions[next_color]:
            if self.board[p] == next_color:
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

        if len(self.moves) % 5 == 0:
            if not self._is_survivable():
                # 詰みが確定している場合は、スコアをどん底に落としてビームから除外
                score -= 5000000

        return score
    
    def _is_survivable(self):
        head = self.snake[0]
        # 胴体を障害物としてセット（頭は探索開始地点なので含めない）
        snake_set = set(self.snake[1:]) 
        
        queue = [head]
        visited = {head}
        reachable = 0
        
        # 全探索すると遅いので、最大でも「自分の長さ」か「20マス」空間があれば安全とみなす
        limit = min(len(self.snake), 20) 
        
        idx = 0
        while idx < len(queue):
            curr = queue[idx]
            idx += 1
            reachable += 1
            
            if reachable >= limit:
                return True
                
            r, c = curr // N, curr % N
            
            for dr, dc in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                nr, nc = r + dr, c + dc
                if 0 <= nr < N and 0 <= nc < N:
                    npos = nr * N + nc
                    if npos not in snake_set and npos not in visited:
                        visited.add(npos)
                        queue.append(npos)
                        
        # 探索しきっても limit に届かなかった場合は、袋小路にいる
        return False

def beam_search(beam_width=30, max_turns=100000, time_limit=1.8):
    initial_snake = tuple((4-i) * N for i in range(5))
    initial_colors = tuple([1, 1, 1, 1, 1])
    initial_state = State(initial_snake, initial_colors, mass_flat, "")
    
    beam = [initial_state]
    best_state = initial_state
    
    for turn in range(max_turns):
        # 【変更】1.8秒超えたら蛇行解を返す
        if time.perf_counter() - start_time > time_limit:
            return make_snake_path()
            
        if not beam:
            break
            
        next_beam = []
        visited = set()
        
        for state in beam:
            if state.matched == M and state.garbage == 0:
                if best_state.matched < M or best_state.garbage > 0 or len(state.moves) < len(best_state.moves):
                    best_state = state
                continue

            hr = state.snake[0] // N
            hc = state.snake[0] % N
            pr = state.snake[1] // N
            pc = state.snake[1] % N

            k = len(state.snake)

            for d_idx in range(4):
                dir_name = DIR_NAMES[d_idx]
                
                nr = hr + (1 if dir_name == 'D' else -1 if dir_name == 'U' else 0)
                nc = hc + (1 if dir_name == 'R' else -1 if dir_name == 'L' else 0)
                
                if not (0 <= nr < N and 0 <= nc < N):
                    continue
                if nr == pr and nc == pc:
                    continue
                    
                npos = nr * N + nc
                
                bite_idx_new = -1
                for old_idx in range(2, k - 2):
                    if state.snake[old_idx] == npos:
                        bite_idx_new = old_idx + 1
                        break
                
                if bite_idx_new != -1:
                    h = bite_idx_new
                    new_snake = (npos,) + state.snake[:h]
                    new_colors = state.colors[:h+1]
                    
                    b_list = list(state.board)
                    for p in range(h + 1, k):
                        b_list[state.snake[p - 1]] = state.colors[p]
                    new_board = tuple(b_list)
                    
                    nxt_state = State(new_snake, new_colors, new_board, state.moves + dir_name)
                else:
                    if npos == state.snake[-1]:
                        eaten_color = 0
                    else:
                        eaten_color = state.board[npos]
                        
                    if eaten_color > 0:
                        new_snake = (npos,) + state.snake
                        new_colors = state.colors + (eaten_color,)
                        
                        b_list = list(state.board)
                        b_list[npos] = 0
                        new_board = tuple(b_list)
                        
                        nxt_state = State(new_snake, new_colors, new_board, state.moves + dir_name)
                    else:
                        new_snake = (npos,) + state.snake[:-1]
                        nxt_state = State(new_snake, state.colors, state.board, state.moves + dir_name)
                
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
ans_moves = beam_search(beam_width=40, time_limit=1.8)

if not ans_moves:
    ans_moves = "U"

for move in ans_moves:
    print(move)