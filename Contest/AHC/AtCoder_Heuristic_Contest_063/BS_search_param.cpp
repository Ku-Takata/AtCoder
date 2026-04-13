#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <cmath>
#include <algorithm>
#include <set>
#include <tuple>
#include <unordered_set>
#include <cstdint>
#include <random>

using namespace std;

mt19937 mt_engine(2525);

// 処理開始時間を記録
chrono::high_resolution_clock::time_point start_time;

int N, M, C;
vector<int> color;
vector<int> mass_flat;

int current_penalty_weight = 2; // ペナルティの重さ（グローバル変数）

const int MAX_N = 16;
int visited_survivable[MAX_N * MAX_N];
int survivable_id = 0;

char DIR_NAMES[] = {'U', 'D', 'L', 'R'};

// 各色の餌の初期位置をあらかじめ分類しておく（高速化の肝）
vector<vector<int>> bait_positions;

int pos_r[MAX_N * MAX_N];
int pos_c[MAX_N * MAX_N];
int adj[MAX_N * MAX_N][4];
int adj_len[MAX_N * MAX_N];

string make_snake_path();

// --- 追加: string moves の丸ごとコピーを防ぐための履歴ツリー ---
struct HistoryNode {
    int parent_id;
    char move;
};
vector<HistoryNode> history_pool;

// --- 変更: 全て vector から固定長配列へ（高速コピーのため） ---
struct State {
    int16_t snake[MAX_N * MAX_N];
    int16_t snake_len;
    
    int16_t colors[MAX_N * MAX_N];
    int16_t colors_len;
    
    int16_t board[MAX_N * MAX_N];
    
    int node_id;
    // --- 変更箇所: int16_t だと 32767 でオーバーフローするため int に戻す ---
    int moves_length;
    
    int16_t matched;
    int16_t garbage;
    long long score;

    bool _is_survivable() {
        int head = snake[0];
        
        survivable_id++;
        
        // 胴体を「すでに訪問済み」として直接マークする
        for (int i = 1; i < snake_len; ++i) {
            visited_survivable[snake[i]] = survivable_id;
        }

        // vectorの毎回の生成をやめ、使い回せるstatic配列にする
        static int queue[MAX_N * MAX_N];
        queue[0] = head;
        visited_survivable[head] = survivable_id;
        
        int reachable = 0;
        int limit = min((int)snake_len, 20); 
        
        int idx = 0;
        int tail = 1; 
        
        int dr[] = {-1, 1, 0, 0};
        int dc[] = {0, 0, -1, 1};
        
        while (idx < tail) {
            int curr = queue[idx++]; 
            reachable++;
            
            if (reachable >= limit) {
                return true;
            }
                
            // 変更箇所: dr, dc の計算と割り算を撤廃し、隣接リストを使う
            for (int i = 0; i < adj_len[curr]; ++i) {
                int npos = adj[curr][i];
                if (visited_survivable[npos] != survivable_id) {
                    visited_survivable[npos] = survivable_id;
                    queue[tail++] = npos; 
                }
            }
        }
        return false;
    }

    long long _calc_score() {
        long long score_val = matched * 1000000LL;
        score_val -= moves_length * 10LL;
        
        if (matched == M && garbage == 0) {
            return score_val + 10000000LL;
        }
            
        int head_pos = snake[0];
        int head_r = pos_r[head_pos];
        int head_c = pos_c[head_pos];

        int next_color = (matched < M) ? color[matched] : colors[colors_len - 1];
        
        int min_dist = 10000;

        for (int p : bait_positions[next_color]) {
            if (board[p] == next_color) {
                int r = pos_r[p];
                int c = pos_c[p];
                int dist = abs(head_r - r) + abs(head_c - c);
                
                int min_r = min(head_r, r);
                int max_r = max(head_r, r);
                int min_c = min(head_c, c);
                int max_c = max(head_c, c);
                
                int obstacle_count = 0;
                for (int i = 1; i < snake_len; ++i) {
                    int sr = pos_r[snake[i]];
                    int sc = pos_c[snake[i]];
                    if (min_r <= sr && sr <= max_r && min_c <= sc && sc <= max_c) {
                        obstacle_count++;
                    }
                }
                
                // グローバル変数 current_penalty_weight を使用
                int perceived_dist = dist + obstacle_count * current_penalty_weight;

                if (perceived_dist < min_dist) {
                    min_dist = perceived_dist;
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
                int target_r = pos_r[target_pos];
                int target_c = pos_c[target_pos];
                
                int dist_to_bite = abs(head_r - target_r) + abs(head_c - target_c);
                
                int min_r = min(head_r, target_r);
                int max_r = max(head_r, target_r);
                int min_c = min(head_c, target_c);
                int max_c = max(head_c, target_c);
                
                int obstacle_count = 0;
                for (int i = 1; i < snake_len; ++i) {
                    if (snake[i] == target_pos) continue; 
                    int sr = pos_r[snake[i]];
                    int sc = pos_c[snake[i]];
                    if (min_r <= sr && sr <= max_r && min_c <= sc && sc <= max_c) {
                        obstacle_count++;
                    }
                }
                
                // 噛みちぎる際にもランダムなペナルティ重みを適用
                int perceived_dist_bite = dist_to_bite + obstacle_count * current_penalty_weight;
                score_val -= perceived_dist_bite * 100LL;
            }
        }

        if (moves_length > 0 && moves_length % 5 == 0) {
            if (!_is_survivable()) {
                score_val -= 5000000LL;
            }
        }

        return score_val;
    }

    void update_match() {
        int m = 0;
        int limit = min((int)colors_len, M);
        while (m < limit && colors[m] == color[m]) {
            m++;
        }
        matched = m;
        garbage = colors_len - m;
    }

    State() : snake_len(0), colors_len(0), node_id(-1), moves_length(0), matched(0), garbage(0), score(0) {}

    // 初期のvectorから配列へ移し替えるための関数
    void init(const vector<int>& s, const vector<int>& c, const vector<int>& b) {
        snake_len = s.size();
        for(int i = 0; i < snake_len; ++i) snake[i] = s[i];
        
        colors_len = c.size();
        for(int i = 0; i < colors_len; ++i) colors[i] = c[i];
        
        for(int i = 0; i < N * N; ++i) board[i] = b[i];
        
        node_id = 0;
        moves_length = 0;
        
        update_match();
        score = _calc_score();
    }
};

using StateKey = tuple<int, int, int, int, int>;

struct StateKeyHash {
    size_t operator()(const StateKey& k) const {
        size_t res = 17;
        res = res * 31 + hash<int>()(get<0>(k));
        res = res * 31 + hash<int>()(get<1>(k));
        res = res * 31 + hash<int>()(get<2>(k));
        res = res * 31 + hash<int>()(get<3>(k));
        res = res * 31 + hash<int>()(get<4>(k));
        return res;
    }
};

string beam_search(int beam_width, int max_turns, double time_limit, int noise_max) {
    history_pool.clear();
    history_pool.reserve(5000000); // これを追加！(再確保によるフリーズを防止)
    history_pool.push_back({-1, ' '}); // root node (ID = 0)

    vector<int> initial_snake(5);
    for (int i = 0; i < 5; ++i) {
        initial_snake[i] = (4 - i) * N;
    }
    vector<int> initial_colors = {1, 1, 1, 1, 1};
    
    State initial_state;
    initial_state.init(initial_snake, initial_colors, mass_flat);
    
    vector<State> beam = {initial_state};
    State best_state = initial_state;
    
    // --- 変更箇所: vectorの代わりに固定長プールとインデックス配列を用意 ---
    State next_beam_pool[250];
    pair<long long, int> next_beam_info[250];
    StateKey next_beam_keys[250]; // --- 追加: 重複判定用のキー配列
    int next_beam_size = 0;
    
    for (int turn = 0; turn < max_turns; ++turn) {
        auto current_time = chrono::high_resolution_clock::now();
        double elapsed_time = chrono::duration<double>(current_time - start_time).count();
        if (elapsed_time > time_limit) {
            return ""; // 失敗の印として空文字を返す
        }
            
        if (beam.empty()) {
            break;
        }
            
        next_beam_size = 0; // next_beam.clear() の代わり
        
        for (State& state : beam) {
            if (state.matched == M && state.garbage == 0) {
                if (best_state.matched < M || best_state.garbage > 0 || state.moves_length < best_state.moves_length) {
                    best_state = state;
                }
                continue;
            }

            // 割り算を撤廃し、事前計算した pos_r, pos_c を使う
            int hr = pos_r[state.snake[0]];
            int hc = pos_c[state.snake[0]];
            int pr = state.snake[1]; // 比較用にインデックスをそのまま取得

            int k = state.snake_len;

            for (int d_idx = 0; d_idx < 4; ++d_idx) {
                char dir_name = DIR_NAMES[d_idx];
                
                // nr, nc を経由せず、条件分岐で直接 npos を計算する
                int npos = -1;
                if (dir_name == 'U' && hr > 0) npos = state.snake[0] - N;
                else if (dir_name == 'D' && hr < N - 1) npos = state.snake[0] + N;
                else if (dir_name == 'L' && hc > 0) npos = state.snake[0] - 1;
                else if (dir_name == 'R' && hc < N - 1) npos = state.snake[0] + 1;

                // 盤面外に出ようとした場合や、直前の首の骨(pr)に逆走した場合はスキップ
                if (npos == -1 || npos == pr) {
                    continue;
                }
                
                // ↓この bite_idx_new の行に繋がります
                int bite_idx_new = -1;
                for (int old_idx = 2; old_idx < k - 2; ++old_idx) {
                    if (state.snake[old_idx] == npos) {
                        bite_idx_new = old_idx + 1;
                        break;
                    }
                }
                
                if (bite_idx_new != -1) {
                    int h = bite_idx_new;
                    
                    // --- 変更箇所: プール内の参照を直接書き換える ---
                    State& nxt_state = next_beam_pool[next_beam_size];
                    nxt_state = state;
                    
                    nxt_state.node_id = history_pool.size();
                    history_pool.push_back({state.node_id, dir_name});
                    nxt_state.moves_length++;
                    
                    // スネークを1マス進める処理
                    for(int i = h; i > 0; --i) nxt_state.snake[i] = state.snake[i-1];
                    nxt_state.snake[0] = npos;
                    nxt_state.snake_len = h + 1;
                    nxt_state.colors_len = h + 1;
                    
                    for (int p = h + 1; p < k; ++p) {
                        nxt_state.board[state.snake[p - 1]] = state.colors[p];
                    }
                    
                    nxt_state.update_match();
                    nxt_state.score = nxt_state._calc_score();
                    // --- 追加: スコアを少し揺らして、同じ状態でも選ばれるルートを変える ---
                    nxt_state.score += mt_engine() % noise_max;
                    // --------------------------------------------------------------------
                    
                    StateKey state_key = make_tuple(nxt_state.snake[0], nxt_state.snake[nxt_state.snake_len - 1], nxt_state.snake_len, nxt_state.matched, nxt_state.garbage);
                    
                    // --- 変更箇所: unordered_set をやめ、配列の線形探索にする ---
                    bool is_dup = false;
                    for (int i = 0; i < next_beam_size; ++i) {
                        if (next_beam_keys[i] == state_key) {
                            is_dup = true;
                            break;
                        }
                    }
                    if (!is_dup) {
                        next_beam_keys[next_beam_size] = state_key;
                        next_beam_info[next_beam_size] = {nxt_state.score, next_beam_size};
                        next_beam_size++;
                    }
                    // --------------------------------------------------------
                } else {
                    int eaten_color;
                    if (npos == state.snake[k - 1]) {
                        eaten_color = 0;
                    } else {
                        eaten_color = state.board[npos];
                    }
                        
                    if (eaten_color > 0) {
                        // --- 変更箇所: プール内の参照を直接書き換える ---
                        State& nxt_state = next_beam_pool[next_beam_size];
                        nxt_state = state;
                        
                        nxt_state.node_id = history_pool.size();
                        history_pool.push_back({state.node_id, dir_name});
                        nxt_state.moves_length++;
                        
                        for(int i = k; i > 0; --i) nxt_state.snake[i] = state.snake[i-1];
                        nxt_state.snake[0] = npos;
                        nxt_state.snake_len = k + 1;
                        
                        nxt_state.colors[k] = eaten_color;
                        nxt_state.colors_len = k + 1;
                        nxt_state.board[npos] = 0;
                        
                        nxt_state.update_match();
                        nxt_state.score = nxt_state._calc_score();
                        // --- 追加: スコアを少し揺らして、同じ状態でも選ばれるルートを変える ---
                        nxt_state.score += mt_engine() % noise_max;
                        // --------------------------------------------------------------------
                        
                        StateKey state_key = make_tuple(nxt_state.snake[0], nxt_state.snake[nxt_state.snake_len - 1], nxt_state.snake_len, nxt_state.matched, nxt_state.garbage);
                    
                        // --- 変更箇所: unordered_set をやめ、配列の線形探索にする ---
                        bool is_dup = false;
                        for (int i = 0; i < next_beam_size; ++i) {
                            if (next_beam_keys[i] == state_key) {
                                is_dup = true;
                                break;
                            }
                        }
                        if (!is_dup) {
                            next_beam_keys[next_beam_size] = state_key;
                            next_beam_info[next_beam_size] = {nxt_state.score, next_beam_size};
                            next_beam_size++;
                        }
                        // --------------------------------------------------------
                    } else {
                        // --- 変更箇所: プール内の参照を直接書き換える ---
                        State& nxt_state = next_beam_pool[next_beam_size];
                        nxt_state = state;
                        
                        nxt_state.node_id = history_pool.size();
                        history_pool.push_back({state.node_id, dir_name});
                        nxt_state.moves_length++;
                        
                        for(int i = k - 1; i > 0; --i) nxt_state.snake[i] = state.snake[i-1];
                        nxt_state.snake[0] = npos;
                        // snake_len はそのまま
                        
                        nxt_state.update_match();
                        nxt_state.score = nxt_state._calc_score();
                        // --- 追加: スコアを少し揺らして、同じ状態でも選ばれるルートを変える ---
                        nxt_state.score += mt_engine() % noise_max;
                        // --------------------------------------------------------------------
                        
                        StateKey state_key = make_tuple(nxt_state.snake[0], nxt_state.snake[nxt_state.snake_len - 1], nxt_state.snake_len, nxt_state.matched, nxt_state.garbage);
                    
                        // --- 変更箇所: unordered_set をやめ、配列の線形探索にする ---
                        bool is_dup = false;
                        for (int i = 0; i < next_beam_size; ++i) {
                            if (next_beam_keys[i] == state_key) {
                                is_dup = true;
                                break;
                            }
                        }
                        if (!is_dup) {
                            next_beam_keys[next_beam_size] = state_key;
                            next_beam_info[next_beam_size] = {nxt_state.score, next_beam_size};
                            next_beam_size++;
                        }
                        // --------------------------------------------------------
                    }
                }
            }
        }
        
        // --- 変更箇所: 構造体を見に行かず、ペアのスコアだけで高速にソートする ---
        if (next_beam_size == 0) {
            break;
        }
            
        sort(next_beam_info, next_beam_info + next_beam_size, [](const pair<long long, int>& a, const pair<long long, int>& b) {
            return a.first > b.first; // ペアの1番目(スコア)だけで比較
        });
        
        beam.clear();
        int keep_count = min(next_beam_size, beam_width);
        for (int i = 0; i < keep_count; ++i) {
            // ペアの2番目(インデックス)を使ってbeamにコピーする
            beam.push_back(next_beam_pool[next_beam_info[i].second]);
        }

        if (beam[0].score > best_state.score) {
            best_state = beam[0];
        }

        if (best_state.matched == M && best_state.garbage == 0) {
            break;
        }
    } // <- for (int turn = 0; turn < max_turns; ++turn) の閉じカッコ
            
    // --- 追加箇所: 探索が終わった時点で餌を全て回収できていなければ、蛇行ルートを返す ---
    if (best_state.matched < M || best_state.garbage > 0) {
        return "";
    }
    // -------------------------------------------------------------------------

    // --- 追加: 最後に履歴ツリーから文字列を復元する ---
    string best_moves = "";
    int curr_node = best_state.node_id;
    while (curr_node > 0) { // 0 はルート（初期状態）
        best_moves += history_pool[curr_node].move;
        curr_node = history_pool[curr_node].parent_id;
    }
    reverse(best_moves.begin(), best_moves.end());
    
    return best_moves;
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

    // 座標と隣接マスの事前計算
    for(int p = 0; p < N * N; ++p) {
        pos_r[p] = p / N;
        pos_c[p] = p % N;
        adj_len[p] = 0;
        int r = pos_r[p];
        int c = pos_c[p];
        if(r > 0) adj[p][adj_len[p]++] = p - N; // U
        if(r < N - 1) adj[p][adj_len[p]++] = p + N; // D
        if(c > 0) adj[p][adj_len[p]++] = p - 1; // L
        if(c < N - 1) adj[p][adj_len[p]++] = p + 1; // R
    }
    
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
    
    bait_positions.resize(C + 1);
    for (int p = 0; p < mass_flat.size(); ++p) {
        int c_val = mass_flat[p];
        if (c_val > 0) {
            bait_positions[c_val].push_back(p);
        }
    }

    string best_ans = "";
    int min_moves = 1000000;
    
    // --- 追加: 難易度別の精鋭パラメータ群（5個ずつに拡張） ---
    // ① 激ムズ用（とにかく迷路を抜けるための大暴れセット）
    vector<pair<int, int>> params_hard = {
        {20, 1500}, {15, 1800}, {25, 1200}, {10, 2000}, {22, 1600}
    };
    // ② 標準用（バランスセット）
    vector<pair<int, int>> params_normal = {
        {35, 500}, {30, 800}, {28, 1000}, {40, 400}, {32, 600}
    };
    // ③ 簡単用（真っ直ぐ進んで手数を極限まで詰めるセット）
    vector<pair<int, int>> params_easy = {
        {50, 150}, {45, 250}, {40, 100}, {50, 300}, {48, 200}
    };
    
    vector<pair<int, int>> current_pool;
    int param_idx = 0;
    
    // ----------------------------------------------------
    // 【第1段階】偵察（プローブ）実行
    // 最初の0.3秒を使って、この盤面の難易度を測る
    // ----------------------------------------------------
    string probe_ans = beam_search(30, 100000, 0.3, 800);
    
    if (probe_ans.empty()) {
        // 0.3秒でゴールできなかった → 「激ムズ盤面」と判定
        current_pool = params_hard;
    } else {
        best_ans = probe_ans;
        min_moves = probe_ans.length();
        
        if (min_moves < 4000) {
            // 余裕で完走し、手数も短い → 「簡単盤面」と判定
            current_pool = params_easy;
        } else {
            // 完走はしたが手数がかかっている → 「標準盤面」と判定
            current_pool = params_normal;
        }
    }

    // ----------------------------------------------------
    // 【第2段階】難易度に合わせたプールで、全力リセマラ
    // ----------------------------------------------------
    while (true) {
        auto current_time = chrono::high_resolution_clock::now();
        double elapsed_time = chrono::duration<double>(current_time - start_time).count();
        if (elapsed_time > 1.8) break; // 1.8秒を使い切るまで無限に回す
        
        // --- 修正: 実験結果に基づき、どんなケースでも最大0.3秒に固定 ---
        double attempt_end_time = elapsed_time + min(0.3, 1.8 - elapsed_time);
        
        int current_beam = current_pool[param_idx].first;
        int current_noise = current_pool[param_idx].second;
        
        param_idx = (param_idx + 1) % current_pool.size(); // 5個のパラメータを順番に回す
        
        string ans = beam_search(current_beam, 100000, attempt_end_time, current_noise);
        
        if (!ans.empty() && ans.length() < min_moves) {
            best_ans = ans;
            min_moves = ans.length();
        }
    }

    // 全てのリセマラで完走できなかった最悪ケースのみ、蛇行ルートを出力
    if (best_ans.empty()) {
        best_ans = make_snake_path();
    }

    for (char move : best_ans) {
        cout << move << "\n";
    }

    return 0;
}