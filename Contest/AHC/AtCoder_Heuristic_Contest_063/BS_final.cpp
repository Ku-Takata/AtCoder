// 完走安定板
// これ以上伸ばせない...
// 絶望のシードガチャ 80位台に残れたらラッキー

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

mt19937 mt_engine(2929);

// 処理開始時間を記録
chrono::high_resolution_clock::time_point start_time;

int N, M, C;
vector<int> color;
vector<int> mass_flat;

// 評価関数のハイパーパラメータ（グローバル変数）
int current_penalty_weight = 2;
long long current_turn_penalty = 50;
long long current_bite_penalty = 100;
long long current_garbage_penalty = 50;

// --- 追加: デバッグ情報伝達用のグローバル変数 ---
int debug_max_turn = 0;
int debug_matched = 0;
int debug_garbage = 0;
int debug_evaluated_nodes = 0;
string debug_fail_reason = "";

const int MAX_N = 16;
int visited_survivable[MAX_N * MAX_N];
int survivable_id = 0;

char DIR_NAMES[] = {'U', 'D', 'L', 'R'};

// 各色の餌の初期位置をあらかじめ分類しておく（高速化のため）
vector<vector<int>> bait_positions;

int pos_r[MAX_N * MAX_N];
int pos_c[MAX_N * MAX_N];
int adj[MAX_N * MAX_N][4];
int adj_len[MAX_N * MAX_N];

string make_snake_path();

// 成功した試行のパラメータを記録するための構造体
struct SuccessRecord {
    int moves;
    int trial_id;
    int beam;
    int noise;
    int pen;
    long long trn;
    long long bte;
    long long grb;
};

// string moves の丸ごとコピーを防ぐための履歴ツリー
struct HistoryNode {
    int parent_id;
    char move;
};
vector<HistoryNode> history_pool;

struct State {
    int16_t snake[MAX_N * MAX_N];
    int16_t snake_len;
    int16_t colors[MAX_N * MAX_N];
    int16_t colors_len;
    int16_t board[MAX_N * MAX_N];

    int node_id;
    int moves_length;

    int16_t matched;
    int16_t garbage;
    long long score;
    int last_match_turn;
    int16_t total_eaten;

    bool _is_survivable() {
        int head = snake[0];

        survivable_id++;

        // 胴体を壁として直接マークする
        for (int i = 1; i < snake_len; ++i) {
            visited_survivable[snake[i]] = survivable_id;
        }

        static int queue[MAX_N * MAX_N];
        queue[0] = head;
        visited_survivable[head] = survivable_id;

        int reachable = 0;
        int limit = min((int)snake_len, 20);

        int idx = 0;
        int tail = 1;

        while (idx < tail) {
            int curr = queue[idx++];
            reachable++;
            if (reachable >= limit) {
                return true;
            }

            // 袋小路になるかどうかを判定、もし空きマスが20マス以上なら安全とし、上でtrueを返す
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
        score_val -= moves_length * current_turn_penalty;

        if (matched == M && garbage == 0) {
            return score_val + 10000000LL;
        }

        // 目的の色が10個集まるまでの序盤は、何でもいいから食べて盤面に空白を作ることを高く評価する
        if (matched < 10) {
            score_val += total_eaten * 500LL;
        }

        // 序盤の詰まり対策：集めた数が少ないほど、パニック(噛みちぎり)への沸点を低くする
        // 要改善: ターン数全部100でもまあまあだから色々検証してもいいかも
        int stagnant_turns = moves_length - last_match_turn;
        int panic_limit = 400;
        if (matched < 5) {
            panic_limit = 100;
        } else if (matched < 20) {
            panic_limit = 200;
        }

        int head_pos = snake[0];
        int head_r = pos_r[head_pos];
        int head_c = pos_c[head_pos];

        bool is_bulldozer = false;

        if (stagnant_turns > panic_limit) {
            int ideal_len = max(4, (int)matched);
            int bite_idx = ideal_len - 2;

            // 噛みたい位置(bite_idx)が尻尾の先端2マス以内だとルール上噛めないため、
            // ヘビの長さが bite_idx + 2 を超えている必要がある
            if (bite_idx > snake_len - 3) {
                // 【モードA: ブルドーザーモード (クラスター突撃)】
                // ゴミを完全に切り落とすための長さが足りない場合、ペナルティを無視して次の正解の密集地へ強引に突進し、食べながら伸ばす。
                is_bulldozer = true;
            } else {
                // 【モードB: 完璧な噛みちぎりモード】
                // 長さが十分に足りたので、正解の色を傷つけず、かつゴミを残さない完璧な位置を噛む
                int target_pos = snake[bite_idx];
                int target_r = pos_r[target_pos];
                int target_c = pos_c[target_pos];
                int dist_to_cut = abs(head_r - target_r) + abs(head_c - target_c);
                score_val -= dist_to_cut * 50000LL;
            }
        }

        int next_color = (matched < M) ? color[matched] : colors[colors_len - 1];

        // クラスター評価用に3手先までの色を取得
        int c1 = (matched + 1 < M) ? color[matched + 1] : -1;
        int c2 = (matched + 2 < M) ? color[matched + 2] : -1;
        int c3 = (matched + 3 < M) ? color[matched + 3] : -1;

        int min_dist = 10000;
        int best_cluster_eval = 1000000;

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

                int perceived_dist = dist + obstacle_count * current_penalty_weight;

                if (perceived_dist < min_dist) {
                    min_dist = perceived_dist;
                }

                if (is_bulldozer) {
                    // 周囲5x5マスに次の色がどれだけあるかカウント
                    int cluster_bonus = 0;
                    for (int dr = -2; dr <= 2; ++dr) {
                        for (int dc = -2; dc <= 2; ++dc) {
                            int nr = r + dr;
                            int nc = c + dc;
                            if (nr >= 0 && nr < N && nc >= 0 && nc < N) {
                                int color_at = board[nr * N + nc];
                                if (color_at == c1 || color_at == c2 || color_at == c3) {
                                    cluster_bonus += 3; // 密集していれば大きくプラス
                                }
                            }
                        }
                    }
                    // 距離からボーナスを引いたものを評価値とする
                    int eval = perceived_dist - cluster_bonus;
                    if (eval < best_cluster_eval) {
                        best_cluster_eval = eval;
                    }
                }
            }
        }

        if (is_bulldozer) {
            if (best_cluster_eval != 1000000) {
                // クラスターに向けて誘導
                score_val -= best_cluster_eval * 50000LL;
            }
        } else {
            if (min_dist != 10000) {
                // 通常時の誘導
                score_val -= min_dist * 100LL;
            }
        }

        if (garbage > 0) {
            // ブルドーザーモード中はゴミを食べるペナルティを強制ゼロにして好き勝手に壁を貫通させる
            long long g_penalty = is_bulldozer ? 0 : current_garbage_penalty;
            score_val -= garbage * g_penalty;
            int bite_idx = max(2, (int)matched - 2);
            if (bite_idx <= snake_len - 3) {
                int target_pos = snake[bite_idx];
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

                // ブルドーザーモード中は、自分の体から離れるペナルティも強制ゼロにする
                long long b_penalty = is_bulldozer ? 0 : current_bite_penalty;
                int perceived_dist_bite = dist_to_bite + obstacle_count * current_penalty_weight;
                score_val -= perceived_dist_bite * b_penalty;
            }
        }

        if (moves_length > 0 && moves_length % 10 == 0) {
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

    State() : snake_len(0), colors_len(0), node_id(-1), moves_length(0), matched(0), garbage(0), score(0), last_match_turn(0), total_eaten(0) {}

    void init(const vector<int>& s, const vector<int>& c, const vector<int>& b) {
        snake_len = s.size();
        for(int i = 0; i < snake_len; ++i) snake[i] = s[i];
        colors_len = c.size();
        for(int i = 0; i < colors_len; ++i) colors[i] = c[i];
        for(int i = 0; i < N * N; ++i) board[i] = b[i];

        node_id = 0;
        moves_length = 0;
        last_match_turn = 0;
        total_eaten = 0;

        update_match();
        score = _calc_score();
    }
};

using StateKey = tuple<int, int, int, int, int, int>;

struct StateKeyHash {
    size_t operator()(const StateKey& k) const {
        size_t res = 17;
        res = res * 31 + hash<int>()(get<0>(k));
        res = res * 31 + hash<int>()(get<1>(k));
        res = res * 31 + hash<int>()(get<2>(k));
        res = res * 31 + hash<int>()(get<3>(k));
        res = res * 31 + hash<int>()(get<4>(k));
        res = res * 31 + hash<int>()(get<5>(k));
        return res;
    }
};

string beam_search(int beam_width, int max_turns, double time_limit, int noise_max) {
    debug_evaluated_nodes = 0;
    debug_fail_reason = "";

    history_pool.clear();
    history_pool.reserve(5000000);
    history_pool.push_back({-1, ' '});

    vector<int> initial_snake(5);
    for (int i = 0; i < 5; ++i) {
        initial_snake[i] = (4 - i) * N;
    }
    vector<int> initial_colors = {1, 1, 1, 1, 1};
    State initial_state;
    initial_state.init(initial_snake, initial_colors, mass_flat);

    vector<State> beam = {initial_state};
    State best_state = initial_state;

    State next_beam_pool[250];
    pair<long long, int> next_beam_info[250];
    unordered_set<StateKey, StateKeyHash> next_beam_keys_set;
    int next_beam_size = 0;
    int last_match_count = 0;
    int last_match_turn = 0;

    for (int turn = 0; turn < max_turns; ++turn) {
        auto current_time = chrono::high_resolution_clock::now();
        double elapsed_time = chrono::duration<double>(current_time - start_time).count();
        if (elapsed_time > time_limit) {
            // 失敗理由と到達状態を記録してリターン
            debug_fail_reason = "Time Out";
            debug_max_turn = best_state.moves_length;
            debug_matched = best_state.matched;
            debug_garbage = best_state.garbage;
            return "";
        }

        if (beam.empty()) {
            debug_fail_reason = "Beam Empty (Dead End)";
            break;
        }

        next_beam_size = 0;
        // 毎ターン重複判定用のセットをクリアする
        next_beam_keys_set.clear();

        // 早期打ち切り
        if (turn - last_match_turn > 500) {
            debug_fail_reason = "Early Stopped (Stuck)";
            break;
        }

        for (State& state : beam) {
            if (state.matched == M && state.garbage == 0) {
                if (best_state.matched < M || best_state.garbage > 0 || state.moves_length < best_state.moves_length) {
                    best_state = state;
                }
                continue;
            }

            // 事前計算した pos_r, pos_c を使う
            int hr = pos_r[state.snake[0]];
            int hc = pos_c[state.snake[0]];
            int pr = state.snake[1];

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

                // 噛みちぎり判定
                int bite_idx_new = -1;
                for (int old_idx = 2; old_idx < k - 2; ++old_idx) {
                    if (state.snake[old_idx] == npos) {
                        bite_idx_new = old_idx + 1;
                        break;
                    }
                }
                if (bite_idx_new != -1) {
                    int h = bite_idx_new;
                    State& nxt_state = next_beam_pool[next_beam_size];
                    nxt_state = state;
                    nxt_state.node_id = history_pool.size();
                    history_pool.push_back({state.node_id, dir_name});
                    nxt_state.moves_length++;

                    // 噛みちぎり時の挙動
                    for(int i = h; i > 0; --i) nxt_state.snake[i] = state.snake[i-1];
                    nxt_state.snake[0] = npos;
                    nxt_state.snake_len = h + 1;
                    nxt_state.colors_len = h + 1;
                    for (int p = h + 1; p < k; ++p) {
                        nxt_state.board[state.snake[p - 1]] = state.colors[p];
                    }
                    // 正解数とゴミの再計算
                    nxt_state.update_match();
                    // 正解が進んだ、もしくはスタック状態で噛みちぎりを実行できたらタイマーリセット
                    if (nxt_state.matched > state.matched || state.moves_length - state.last_match_turn > 400) {
                        nxt_state.last_match_turn = nxt_state.moves_length;
                    }

                    nxt_state.score = nxt_state._calc_score();
                    // スコアを少し揺らして、同じ状態でも選ばれるルートを変える
                    nxt_state.score += mt_engine() % noise_max;
                    debug_evaluated_nodes++;

                    int mid_idx = nxt_state.snake_len / 2;
                    StateKey state_key = make_tuple(nxt_state.snake[0], nxt_state.snake[nxt_state.snake_len - 1], nxt_state.snake_len, nxt_state.matched, nxt_state.garbage, nxt_state.snake[mid_idx]);

                    // ルートの重複判定
                    if (next_beam_keys_set.insert(state_key).second) {
                        next_beam_info[next_beam_size] = {nxt_state.score, next_beam_size};
                        next_beam_size++;
                    }
                    } else {
                    int eaten_color;
                    if (npos == state.snake[k - 1]) {
                        eaten_color = 0;
                    } else {
                        eaten_color = state.board[npos];
                    }
                    // 餌を食べた時の処理
                    if (eaten_color > 0) {
                        State& nxt_state = next_beam_pool[next_beam_size];
                        nxt_state = state;
                        // 履歴追加、ターン数増加、食べた総数増加
                        nxt_state.node_id = history_pool.size();
                        history_pool.push_back({state.node_id, dir_name});
                        nxt_state.moves_length++;
                        nxt_state.total_eaten++;
                        // 体を伸ばす
                        for(int i = k; i > 0; --i) nxt_state.snake[i] = state.snake[i-1];
                        nxt_state.snake[0] = npos;
                        nxt_state.snake_len = k + 1;
                        // 餌を胴体に移動
                        nxt_state.colors[k] = eaten_color;
                        nxt_state.colors_len = k + 1;
                        nxt_state.board[npos] = 0;

                        nxt_state.update_match();

                        // マッチが進んだ時のみタイマーリセット
                        if (nxt_state.matched > state.matched) {
                            nxt_state.last_match_turn = nxt_state.moves_length;
                        }

                        nxt_state.score = nxt_state._calc_score();
                        // スコアを少し揺らして、同じ状態でも選ばれるルートを変える
                        nxt_state.score += mt_engine() % noise_max;
                        int mid_idx = nxt_state.snake_len / 2;
                        StateKey state_key = make_tuple(nxt_state.snake[0], nxt_state.snake[nxt_state.snake_len - 1], nxt_state.snake_len, nxt_state.matched, nxt_state.garbage, nxt_state.snake[mid_idx]);
                        if (next_beam_keys_set.insert(state_key).second) {
                            next_beam_info[next_beam_size] = {nxt_state.score, next_beam_size};
                            next_beam_size++;
                        }
                    // 餌を食べなかった時の処理
                    } else {
                        State& nxt_state = next_beam_pool[next_beam_size];
                        nxt_state = state;
                        nxt_state.node_id = history_pool.size();
                        history_pool.push_back({state.node_id, dir_name});
                        nxt_state.moves_length++;

                        for(int i = k - 1; i > 0; --i) nxt_state.snake[i] = state.snake[i-1];
                        nxt_state.snake[0] = npos;

                        nxt_state.update_match();

                        // マッチが進んだ時のみタイマーリセット
                        if (nxt_state.matched > state.matched) {
                            nxt_state.last_match_turn = nxt_state.moves_length;
                        }

                        nxt_state.score = nxt_state._calc_score();
                        // スコアを少し揺らして、同じ状態でも選ばれるルートを変える
                        nxt_state.score += mt_engine() % noise_max;
                        int mid_idx = nxt_state.snake_len / 2;
                        StateKey state_key = make_tuple(nxt_state.snake[0], nxt_state.snake[nxt_state.snake_len - 1], nxt_state.snake_len, nxt_state.matched, nxt_state.garbage, nxt_state.snake[mid_idx]);
                        if (next_beam_keys_set.insert(state_key).second) {
                            next_beam_info[next_beam_size] = {nxt_state.score, next_beam_size};
                            next_beam_size++;
                        }
                    }
                }
            }
        }
        if (next_beam_size == 0) {
            debug_fail_reason = "Beam Empty (Dead End)";
            break;
        }
        sort(next_beam_info, next_beam_info + next_beam_size, [](const pair<long long, int>& a, const pair<long long, int>& b) {
            return a.first > b.first;
        });

        beam.clear();
        int keep_count = min(next_beam_size, beam_width);
        for (int i = 0; i < keep_count; ++i) {
            beam.push_back(next_beam_pool[next_beam_info[i].second]);
        }

        if (beam[0].score > best_state.score) {
            best_state = beam[0];
        }

        // 最後に餌を食ったターンを更新
        if (best_state.matched > last_match_count) {
            last_match_count = best_state.matched;
            last_match_turn = turn;
        }

        if (best_state.matched == M && best_state.garbage == 0) {
            break;
        }
    }
    // 最終的なベスト状態の情報を記録
    debug_max_turn = best_state.moves_length;
    debug_matched = best_state.matched;
    debug_garbage = best_state.garbage;

    // 探索が終わった時点で餌を全て回収できていなければ、蛇行ルートを返す
    if (best_state.matched < M || best_state.garbage > 0) {
        // ベストを越えなかったため、打ち切りであることを表示する
        if (debug_fail_reason == "") {
            if (max_turns != 100000) {
                debug_fail_reason = "Pruned by Best Score";
            } else {
                debug_fail_reason = "Not Finished";
            }
        }
        return "";
    }

    // 最後に履歴ツリーから文字列を復元する
    string best_moves = "";
    int curr_node = best_state.node_id;
    while (curr_node > 0) {
        best_moves += history_pool[curr_node].move;
        curr_node = history_pool[curr_node].parent_id;
    }
    reverse(best_moves.begin(), best_moves.end());

    return best_moves;
}

// シンプルな蛇行
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

    // 成功した試行を保存しておくリスト
    vector<SuccessRecord> success_trials;

    // ランダム生成のための閾値を変数として用意
    int beam_min = 30, beam_max = 50;
    int noise_min = 500, noise_max = 2000;
    int trn_min, trn_max, bte_min, bte_max, grb_min, grb_max;

    // ----------------------------------------------------
    // 【第1段階】偵察
    // ----------------------------------------------------
    cerr << "=== [Phase 1] Probe Start (0.3s) ===\n";
    string probe_ans = beam_search(30, 100000, 0.3, 500);

    if (probe_ans.empty()) {
        cerr << "Difficulty   : HARD\n";
        // 激ムズ用: 完走を最優先。スパルタ度を少し緩め、ゴミも少し許容する。
        beam_min = 30; beam_max = 50;
        noise_min = 300; noise_max = 2000;
        trn_min = 10; trn_max = 300;
        bte_min = 10; bte_max = 500;
        grb_min = 10; grb_max = 100;
    } else {
        best_ans = probe_ans;
        min_moves = probe_ans.length();

        if (min_moves < 2000) {
            cerr << "Difficulty   : EASY\n";
            // 簡単用: 究極のスパルタ設定。最短距離を強制し、ゴミは許さない。
            beam_min = 40; beam_max = 70;
            noise_min = 100; noise_max = 500;
            trn_min = 50; trn_max = 300;
            bte_min = 50; bte_max = 500;
            grb_min = 10; grb_max = 100;
        } else {
            cerr << "Difficulty   : NORMAL\n";
            // 標準用: データ分析で強かった黄金比の範囲。
            beam_min = 20; beam_max = 60;
            noise_min = 200; noise_max = 1200;
            trn_min = 30; trn_max = 300;
            bte_min = 30; bte_max = 500;
            grb_min = 10; grb_max = 100;
        }
    }
    cerr << "====================================\n\n";
    cerr << "=== [Phase 2] Main Search Start ===\n";

    // ----------------------------------------------------
    // 【第2段階】難易度に合わせた範囲でランダムリセマラ
    // ----------------------------------------------------
    int trial_count = 1;
    while (true) {
        auto current_time = chrono::high_resolution_clock::now();
        double elapsed_time = chrono::duration<double>(current_time - start_time).count();
        if (elapsed_time > 1.95) break;

        double attempt_end_time = elapsed_time + min(0.3, 1.95 - elapsed_time);

        // パラメータランダムガチャ
        int current_beam = beam_min + (mt_engine() % (beam_max - beam_min + 1));
        int current_noise = noise_min + (mt_engine() % (noise_max - noise_min + 1));
        current_penalty_weight = 1 + (mt_engine() % 3);
        current_turn_penalty = trn_min + (mt_engine() % (trn_max - trn_min + 1));
        current_bite_penalty = bte_min + (mt_engine() % (bte_max - bte_min + 1));
        current_garbage_penalty = grb_min + (mt_engine() % (grb_max - grb_min + 1));
        // 実行結果を一意に決める
        // mt_engine.seed(2525 + trial_count);

        auto start_search = chrono::high_resolution_clock::now();
        string ans = beam_search(current_beam, min_moves, attempt_end_time, current_noise);
        auto end_search = chrono::high_resolution_clock::now();
        int search_ms = chrono::duration_cast<chrono::milliseconds>(end_search - start_search).count();

        // デバッグ表示
        cerr << "Trial " << trial_count++ << " | Beam: " << current_beam << " | Noise: " << current_noise << " | Pen: " << current_penalty_weight;
        cerr << " | Trn: " << current_turn_penalty << " | Bte: " << current_bite_penalty << " | Grb: " << current_garbage_penalty;
        cerr << " | Time: " << search_ms << "ms | Nodes: " << debug_evaluated_nodes;

        if (ans.empty()) {
            cerr << " | Result: Failed [" << debug_fail_reason << "] (Turn: " << debug_max_turn << ", Matched: " << debug_matched << "/" << M << ", Garbage: " << debug_garbage << ")\n";
        } else {
            cerr << " | Result: " << ans.length() << " moves";
            if (ans.length() < min_moves) {
                best_ans = ans;
                min_moves = ans.length();
                cerr << "  <-- Best Updated!";
            }
            cerr << "\n";

            // 完走できた場合、その結果とパラメータを保存する
            success_trials.push_back({
                (int)ans.length(),
                trial_count - 1,
                current_beam, current_noise, current_penalty_weight,
                current_turn_penalty, current_bite_penalty, current_garbage_penalty
            });
        }
    }
    cerr << "====================================\n";

    // ランキング形式での表示処理
    if (!success_trials.empty()) {
        // ターン数の昇順でソート
        sort(success_trials.begin(), success_trials.end(), [](const SuccessRecord& a, const SuccessRecord& b) {
            return a.moves < b.moves;
        });

        cerr << "\n=== [Ranking of Successful Trials] ===\n";
        for (int i = 0; i < success_trials.size(); ++i) {
            const auto& rec = success_trials[i];
            cerr << "Rank " << i + 1 << " (" << rec.moves << " moves) [Trial " << rec.trial_id << "]\n";
            cerr << "  -> Beam: " << rec.beam << ", Noise: " << rec.noise << ", Pen: " << rec.pen
                 << ", Trn: " << rec.trn << ", Bte: " << rec.bte << ", Grb: " << rec.grb << "\n";
        }
        cerr << "======================================\n\n";
    } else {
        cerr << "\n=== [Ranking of Successful Trials] ===\n";
        cerr << "  No successful trials found.\n";
        cerr << "======================================\n\n";
    }

    cerr << "Final Best Moves: " << (best_ans.empty() ? "Failed" : to_string(best_ans.length())) << "\n";

    if (best_ans.empty()) {
        best_ans = make_snake_path();
    }

    for (char move : best_ans) {
        cout << move << "\n";
    }

    return 0;
}

/* ハイパーパラメータ一覧

ランダムサーチのパラメータ
beam_min / beam_max
    問題の難易度に応じてbeam幅の閾値を変えている。
noise_min / noise_max
    問題の難易度に応じてノイズの閾値を変えている。
trn_min / trn_max
    大きくすると、最短距離を目指しがち(突っ込みがち)になる。小さくすると、遠回りしてでも安全な道を選びがちになる。
bte_min / bte_max
    自分を噛みちぎるためのターゲットの餌の位置から離れることへのペナルティ。
grb_min / grb_max
    ゴミ（違う色）を抱え込むことへのペナルティ。
current_penalty_weight
    胴体を壁として認識し、迂回しようとする強さ。

通常モードのパラメータ（固定値）
min_dist * 100LL
    ターゲットへの距離ペナルティ。餌に近づくための引力の強さ。
matched * 1000000LL
    正解の餌を食べた時の莫大な進行ボーナス。
total_eaten * 500LL
    matched < 10の序盤限定でゴミでも何でも食べたときに与えられる盤面開拓ボーナス。
min_moves < 2000
    問題難易度を判定している。最初の固定のパラメータでターン数が2000ターン未満ならEasy、2000ターン以上ならNormal、解けないならHardと判定している。

別モードのパラメータ（固定値）

密集地計算
dr = -2; dr <= 2
dc = -2; dc <= 2
    密集地（目標の色の餌の群生地）の範囲。±2の範囲、つまり5*5の範囲を密集地として捉える
c1,c2,c3
    3手先までを密集地ボーナスの対象としている。

パニックモード
panic_limit
    スタック（目標の色が進められていない状態）を何ターン許容するか。matchedの数に応じて変化（100, 200, 400）。指定したターンを越えるとパニックモードに移り、適当な動きをする。

ブルドーザーモード
cluster_bonus += 3
    ブルドーザーモード時、周囲5x5マスに未来のターゲットがあった場合に与えられるボーナス値。
best_cluster_eval * 50000LL
    ブルドーザーモード時の、密集地へ向かう引力の強さ。

モードB（安全な噛みちぎりモード）
target_idx = (snake_len * 3) / 4（現在撤廃、ゴミを残す可能性や正解ごと噛みちぎるのは多様性は増やすが、ターン数は増える傾向があるため）
    モードBで噛みちぎる胴体の位置。
dist_to_cut * 50000LL
    モードBで尻尾へ向かう引力の強さ

生存確認、打ち切り制限（固定値）
moves_length % 10 == 0
    BFSを用いた生存確認（空間チェック）を行う頻度。10ターンに1回。
limit = min((int)snake_len, 20)
    生存確認で何マス空きがあれば安全とみなすかの閾値。20マスあればOK。
-5000000LL
    生存確認で20マス未満しか空きがなかった時の即死ペナルティ。
1200
    スタックが何ターン続いているかを計算し任意のターン数を越えた場合、強制終了する。

*/