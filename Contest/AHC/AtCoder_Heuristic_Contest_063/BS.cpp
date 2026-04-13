#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <chrono>
#include <unordered_set>
#include <algorithm>

using namespace std;

// 処理開始時間を記録するためのグローバル変数
auto start_time_clock = chrono::high_resolution_clock::now();

inline double get_time() {
    auto now = chrono::high_resolution_clock::now();
    return chrono::duration<double>(now - start_time_clock).count();
}

int N, M, C;
vector<int> color;
vector<int> mass_flat;
vector<vector<int>> bait_positions;
char DIR_NAMES[4] = {'U', 'D', 'L', 'R'};

struct State {
    vector<int> snake;
    vector<int> colors;
    vector<int> board;
    string moves;
    int matched;
    int garbage;
    int score;

    State() {}

    State(const vector<int>& _snake, const vector<int>& _colors, const vector<int>& _board, const string& _moves) {
        snake = _snake;
        colors = _colors;
        board = _board;
        moves = _moves;

        int m = 0;
        int limit = min((int)colors.size(), M);
        while (m < limit && colors[m] == color[m]) {
            m++;
        }
        matched = m;
        garbage = (int)colors.size() - m;
        
        score = _calc_score();
    }

    int _calc_score() {
        int s = matched * 1000000;
        s -= (int)moves.length() * 10;
        
        if (matched == M && garbage == 0) {
            return s + 10000000;
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
            s -= min_dist * 100;
        }
            
        if (garbage > 0) {
            s -= garbage * 50;
            if (matched >= 2) {
                int target_pos = snake[matched - 2];
                int tr = target_pos / N;
                int tc = target_pos % N;
                int dist_to_bite = abs(head_r - tr) + abs(head_c - tc);
                s -= dist_to_bite * 100;
            }
        }

        return s;
    }
};

// unordered_set で重複判定を行うためのハッシュ設定
struct StateKey {
    int head;
    int tail;
    int length;
    int matched;
    int garbage;

    bool operator==(const StateKey& other) const {
        return head == other.head && tail == other.tail && length == other.length &&
               matched == other.matched && garbage == other.garbage;
    }
};

struct KeyHash {
    size_t operator()(const StateKey& k) const {
        size_t res = 17;
        res = res * 31 + hash<int>()(k.head);
        res = res * 31 + hash<int>()(k.tail);
        res = res * 31 + hash<int>()(k.length);
        res = res * 31 + hash<int>()(k.matched);
        res = res * 31 + hash<int>()(k.garbage);
        return res;
    }
};

string beam_search(int beam_width = 30, int max_turns = 100000, double time_limit = 1.8) {
    vector<int> initial_snake;
    for (int i = 0; i < 5; ++i) {
        initial_snake.push_back((4 - i) * N);
    }
    vector<int> initial_colors = {1, 1, 1, 1, 1};
    State initial_state(initial_snake, initial_colors, mass_flat, "");
    
    vector<State> beam;
    beam.push_back(initial_state);
    State best_state = initial_state;
    
    for (int turn = 0; turn < max_turns; ++turn) {
        // 【TLE回避】1.8秒経過したら強制的にループを抜ける
        if (get_time() > time_limit) {
            break;
        }
            
        if (beam.empty()) {
            break;
        }
            
        vector<State> next_beam;
        unordered_set<StateKey, KeyHash> visited;
        
        for (const State& state : beam) {
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
                    for (int i = 0; i < h; ++i) new_snake.push_back(state.snake[i]);
                    
                    vector<int> new_colors;
                    for (int i = 0; i < h + 1; ++i) new_colors.push_back(state.colors[i]);
                    
                    vector<int> new_board = state.board;
                    for (int p = h + 1; p < k; ++p) {
                        new_board[state.snake[p - 1]] = state.colors[p];
                    }
                    
                    State nxt_state(new_snake, new_colors, new_board, state.moves + dir_name);
                    StateKey key = {nxt_state.snake[0], nxt_state.snake.back(), (int)nxt_state.snake.size(), nxt_state.matched, nxt_state.garbage};
                    
                    if (visited.find(key) == visited.end()) {
                        visited.insert(key);
                        next_beam.push_back(nxt_state);
                    }
                } else {
                    int eaten_color = 0;
                    if (npos != state.snake.back()) {
                        eaten_color = state.board[npos];
                    }
                        
                    if (eaten_color > 0) {
                        vector<int> new_snake;
                        new_snake.push_back(npos);
                        for (int x : state.snake) new_snake.push_back(x);
                        
                        vector<int> new_colors = state.colors;
                        new_colors.push_back(eaten_color);
                        
                        vector<int> new_board = state.board;
                        new_board[npos] = 0;
                        
                        State nxt_state(new_snake, new_colors, new_board, state.moves + dir_name);
                        StateKey key = {nxt_state.snake[0], nxt_state.snake.back(), (int)nxt_state.snake.size(), nxt_state.matched, nxt_state.garbage};
                        
                        if (visited.find(key) == visited.end()) {
                            visited.insert(key);
                            next_beam.push_back(nxt_state);
                        }
                    } else {
                        vector<int> new_snake;
                        new_snake.push_back(npos);
                        for (size_t i = 0; i < state.snake.size() - 1; ++i) {
                            new_snake.push_back(state.snake[i]);
                        }
                        
                        State nxt_state(new_snake, state.colors, state.board, state.moves + dir_name);
                        StateKey key = {nxt_state.snake[0], nxt_state.snake.back(), (int)nxt_state.snake.size(), nxt_state.matched, nxt_state.garbage};
                        
                        if (visited.find(key) == visited.end()) {
                            visited.insert(key);
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
        
        beam.clear();
        for (size_t i = 0; i < min((size_t)beam_width, next_beam.size()); ++i) {
            beam.push_back(next_beam[i]);
        }
        
        if (beam[0].score > best_state.score) {
            best_state = beam[0];
        }
            
        if (best_state.matched == M && best_state.garbage == 0) {
            break;
        }
    }
            
    return best_state.moves;
}

int main() {
    // 入出力を高速化
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    if (!(cin >> N >> M >> C)) return 0;

    color.resize(M);
    for (int i = 0; i < M; ++i) {
        cin >> color[i];
    }

    mass_flat.resize(N * N);
    for (int i = 0; i < N * N; ++i) {
        cin >> mass_flat[i];
    }

    bait_positions.resize(C + 1);
    for (int p = 0; p < N * N; ++p) {
        if (mass_flat[p] > 0) {
            bait_positions[mass_flat[p]].push_back(p);
        }
    }

    // ビーム幅を少し絞ることで、時間内に最後まで到達しやすくしています
    string ans_moves = beam_search(30, 100000, 1.8);

    if (ans_moves.empty()) {
        ans_moves = "U";
    }

    for (char move : ans_moves) {
        cout << move << "\n";
    }

    return 0;
}