#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <cmath>
#include <algorithm>
#include <set>
#include <tuple>

using namespace std;

// 処理開始時間を記録
chrono::high_resolution_clock::time_point start_time;

int N, M, C;
vector<int> color;
vector<int> mass_flat;

char DIR_NAMES[] = {'U', 'D', 'L', 'R'};

// 各色の餌の初期位置をあらかじめ分類しておく（高速化の肝）
vector<vector<int>> bait_positions;

string make_snake_path();

struct State {
    vector<int> snake;
    vector<int> colors;
    vector<int> board;
    string moves;
    int matched;
    int garbage;
    long long score;

    bool _is_survivable() {
        int head = snake[0];
        // 胴体を障害物としてセット（頭は探索開始地点なので含めない）
        vector<bool> snake_set(N * N, false);
        for (size_t i = 1; i < snake.size(); ++i) {
            snake_set[snake[i]] = true;
        }

        vector<int> queue;
        queue.push_back(head);
        
        vector<bool> visited(N * N, false);
        visited[head] = true;
        
        int reachable = 0;
        
        // 全探索すると遅いので、最大でも「自分の長さ」か「20マス」空間があれば安全とみなす
        int limit = min((int)snake.size(), 20); 
        
        int idx = 0;
        int dr[] = {-1, 1, 0, 0};
        int dc[] = {0, 0, -1, 1};
        
        while (idx < queue.size()) {
            int curr = queue[idx++];
            reachable++;
            
            if (reachable >= limit) {
                return true;
            }
                
            int r = curr / N;
            int c = curr % N;
            
            for (int i = 0; i < 4; ++i) {
                int nr = r + dr[i];
                int nc = c + dc[i];
                if (0 <= nr && nr < N && 0 <= nc && nc < N) {
                    int npos = nr * N + nc;
                    if (!snake_set[npos] && !visited[npos]) {
                        visited[npos] = true;
                        queue.push_back(npos);
                    }
                }
            }
        }
        // 探索しきっても limit に届かなかった場合は、袋小路にいる
        return false;
    }

    long long _calc_score() {
        long long score_val = matched * 1000000LL;
        score_val -= moves.length() * 10LL;
        
        if (matched == M && garbage == 0) {
            return score_val + 10000000LL;
        }
            
        int head_pos = snake[0];
        int head_r = head_pos / N;
        int head_c = head_pos % N;

        int next_color = (matched < M) ? color[matched] : color.back();
        
        int min_dist = 10000;
        // 盤面全体ではなく、その色が存在する座標リストだけをループする
        for (int p : bait_positions[next_color]) {
            if (board[p] == next_color) {
                int r = p / N;
                int c = p % N;
                int dist = abs(head_r - r) + abs(head_c - c);
                if (dist < min_dist) {
                    min_dist = dist;
                }
            }
        }
                        
        if (min_dist != 10000) {
            score_val -= min_dist * 100LL;
        }
            
        if (garbage > 0) {
            score_val -= garbage * 50LL;
            if (matched >= 2) {
                int target_pos = snake[matched - 2];
                int tr = target_pos / N;
                int tc = target_pos % N;
                int dist_to_bite = abs(head_r - tr) + abs(head_c - tc);
                score_val -= dist_to_bite * 100LL;
            }
        }

        if (moves.length() % 5 == 0) {
            if (!_is_survivable()) {
                // 詰みが確定している場合は、スコアをどん底に落としてビームから除外
                score_val -= 5000000LL;
            }
        }

        return score_val;
    }

    State() {}

    State(vector<int> snake_, vector<int> colors_, vector<int> board_, string moves_) 
        : snake(snake_), colors(colors_), board(board_), moves(moves_) {
        
        int m = 0;
        int limit = min((int)colors.size(), M);
        while (m < limit && colors[m] == color[m]) {
            m++;
        }
        matched = m;
        garbage = colors.size() - m;
        
        score = _calc_score();
    }
};

using StateKey = tuple<int, int, int, int, int>;

string beam_search(int beam_width = 30, int max_turns = 100000, double time_limit = 1.8) {
    vector<int> initial_snake(5);
    for (int i = 0; i < 5; ++i) {
        initial_snake[i] = (4 - i) * N;
    }
    vector<int> initial_colors = {1, 1, 1, 1, 1};
    State initial_state(initial_snake, initial_colors, mass_flat, "");
    
    vector<State> beam = {initial_state};
    State best_state = initial_state;
    
    for (int turn = 0; turn < max_turns; ++turn) {
        // 【変更】1.8秒超えたら蛇行解を返す
        auto current_time = chrono::high_resolution_clock::now();
        double elapsed_time = chrono::duration<double>(current_time - start_time).count();
        if (elapsed_time > time_limit) {
            return make_snake_path();
        }
            
        if (beam.empty()) {
            break;
        }
            
        vector<State> next_beam;
        set<StateKey> visited;
        
        for (State& state : beam) {
            if (state.matched == M && state.garbage == 0) {
                if (best_state.matched < M || best_state.garbage > 0 || state.moves.length() < best_state.moves.length()) {
                    best_state = state;
                }
                continue;
            }

            int hr = state.snake[0] / N;
            int hc = state.snake[0] % N;
            int pr = state.snake[1] / N;
            int pc = state.snake[1] % N;

            int k = state.snake.size();

            for (int d_idx = 0; d_idx < 4; ++d_idx) {
                char dir_name = DIR_NAMES[d_idx];
                
                int nr = hr + (dir_name == 'D' ? 1 : (dir_name == 'U' ? -1 : 0));
                int nc = hc + (dir_name == 'R' ? 1 : (dir_name == 'L' ? -1 : 0));
                
                if (!(0 <= nr && nr < N && 0 <= nc && nc < N)) {
                    continue;
                }
                if (nr == pr && nc == pc) {
                    continue;
                }
                    
                int npos = nr * N + nc;
                
                int bite_idx_new = -1;
                for (int old_idx = 2; old_idx < k - 2; ++old_idx) {
                    if (state.snake[old_idx] == npos) {
                        bite_idx_new = old_idx + 1;
                        break;
                    }
                }
                
                if (bite_idx_new != -1) {
                    int h = bite_idx_new;
                    
                    vector<int> new_snake;
                    new_snake.push_back(npos);
                    new_snake.insert(new_snake.end(), state.snake.begin(), state.snake.begin() + h);
                    
                    vector<int> new_colors(state.colors.begin(), state.colors.begin() + h + 1);
                    
                    vector<int> new_board = state.board;
                    for (int p = h + 1; p < k; ++p) {
                        new_board[state.snake[p - 1]] = state.colors[p];
                    }
                    
                    State nxt_state(new_snake, new_colors, new_board, state.moves + dir_name);
                    
                    StateKey state_key = make_tuple(nxt_state.snake.front(), nxt_state.snake.back(), nxt_state.snake.size(), nxt_state.matched, nxt_state.garbage);
                    if (visited.find(state_key) == visited.end()) {
                        visited.insert(state_key);
                        next_beam.push_back(nxt_state);
                    }
                } else {
                    int eaten_color;
                    if (npos == state.snake.back()) {
                        eaten_color = 0;
                    } else {
                        eaten_color = state.board[npos];
                    }
                        
                    if (eaten_color > 0) {
                        vector<int> new_snake;
                        new_snake.push_back(npos);
                        new_snake.insert(new_snake.end(), state.snake.begin(), state.snake.end());
                        
                        vector<int> new_colors = state.colors;
                        new_colors.push_back(eaten_color);
                        
                        vector<int> new_board = state.board;
                        new_board[npos] = 0;
                        
                        State nxt_state(new_snake, new_colors, new_board, state.moves + dir_name);
                        
                        StateKey state_key = make_tuple(nxt_state.snake.front(), nxt_state.snake.back(), nxt_state.snake.size(), nxt_state.matched, nxt_state.garbage);
                        if (visited.find(state_key) == visited.end()) {
                            visited.insert(state_key);
                            next_beam.push_back(nxt_state);
                        }
                    } else {
                        vector<int> new_snake;
                        new_snake.push_back(npos);
                        new_snake.insert(new_snake.end(), state.snake.begin(), state.snake.end() - 1);
                        
                        State nxt_state(new_snake, state.colors, state.board, state.moves + dir_name);
                        
                        StateKey state_key = make_tuple(nxt_state.snake.front(), nxt_state.snake.back(), nxt_state.snake.size(), nxt_state.matched, nxt_state.garbage);
                        if (visited.find(state_key) == visited.end()) {
                            visited.insert(state_key);
                            next_beam.push_back(nxt_state);
                        }
                    }
                }
            }
        }
        
        if (next_beam.empty()) {
            break;
        }
            
        sort(next_beam.begin(), next_beam.end(), [](const State& a, const State& b) {
            return a.score > b.score;
        });
        
        if (next_beam.size() > beam_width) {
            next_beam.resize(beam_width);
        }
        beam = next_beam;
        
        if (beam[0].score > best_state.score) {
            best_state = beam[0];
        }
            
        if (best_state.matched == M && best_state.garbage == 0) {
            break;
        }
    }
            
    return best_state.moves;
}

string make_snake_path() {
    string moves = "";
    
    int r = 4, c = 0;
    bool direction_down = true;
    
    while (c < N - 1) {
        if (direction_down) {
            while (r < N - 1) {
                moves += 'D';
                r += 1;
            }
        } else {
            while (r > 0) {
                moves += 'U';
                r -= 1;
            }
        }
        
        moves += 'R';
        c += 1;
        direction_down = !direction_down;
    }

    if (direction_down) {
        while (r < N - 1) {
            moves += 'D';
            r += 1;
        }
    } else {
        while (r > 0) {
            moves += 'U';
            r -= 1;
        }
    }

    return moves;
}

int main() {
    // 処理開始時間を記録
    start_time = chrono::high_resolution_clock::now();
    
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    if (!(cin >> N >> M >> C)) return 0;
    
    color.resize(M);
    for (int i = 0; i < M; ++i) {
        cin >> color[i];
    }
    
    mass_flat.reserve(N * N);
    for (int i = 0; i < N * N; ++i) {
        int c_val;
        cin >> c_val;
        mass_flat.push_back(c_val);
    }
    
    // 各色の餌の初期位置をあらかじめ分類しておく（高速化の肝）
    bait_positions.resize(C + 1);
    for (int p = 0; p < mass_flat.size(); ++p) {
        int c_val = mass_flat[p];
        if (c_val > 0) {
            bait_positions[c_val].push_back(p);
        }
    }

    // タイムリミットは1.8
    string ans_moves = beam_search(40, 100000, 1.8);

    if (ans_moves.empty()) {
        ans_moves = "U";
    }

    for (char move : ans_moves) {
        cout << move << "\n";
    }

    return 0;
}