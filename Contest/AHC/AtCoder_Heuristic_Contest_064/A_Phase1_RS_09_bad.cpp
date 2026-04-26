#include <iostream>
#include <vector>
#include <deque>
#include <algorithm>
#include <chrono>

using namespace std;

// 高速な時間計測用クラス
struct Timer {
    chrono::high_resolution_clock::time_point st;
    Timer() { st = chrono::high_resolution_clock::now(); }
    double elapsed() {
        auto ed = chrono::high_resolution_clock::now();
        return chrono::duration_cast<chrono::duration<double>>(ed - st).count();
    }
};

// 高速な乱数生成器 (XorShift)
uint32_t xor32(uint32_t& x) {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

struct Op {
    int type, i, j, k;
};

// ビームサーチ用の状態構造体
struct State {
    vector<deque<int>> D, S;
    vector<Op> seq_ops;
    int score;
    bool operator<(const State& o) const {
        return score > o.score; // 降順ソート用
    }
};

// 盤面の評価関数 (待避線Sの美しさと、消費ターン数を評価)
int calc_eval(int R, const vector<deque<int>>& S, int turns) {
    int score = 0;
    score -= turns * 10000; // ターン数は少ないほど良い
    
    int moved = 0;
    for (int j = 0; j < R; ++j) {
        moved += S[j].size();
        int depth = 0;
        // rbegin() はSの「奥底(末尾)」から「手前(先頭)」に向かってイテレートする
        for (auto it = S[j].rbegin(); it != S[j].rend(); ++it) {
            int step_val = (*it) % 10;
            // 奥底(depth=0)に、大きい数字(step_val=9)があるほど高得点
            score += step_val * (10 - depth) * 10;
            depth++;
        }
    }
    score += moved * 1000; // より多くの車両をSに送れているか
    return score;
}

// 1回のビームサーチを実行し、Phase 1を完了させた最高状態を返す
State run_beam_search(int R, const vector<deque<int>>& init_D, const vector<deque<int>>& init_S, uint32_t& rng) {
    int BEAM_WIDTH = 5; // ビーム幅（ここを増やすと1回の探索が重く・賢くなる）
    int MAX_EXPAND = 20;  // 1つの状態から展開する次の一手の最大数

    vector<State> beam;
    State init_st;
    init_st.D = init_D; init_st.S = init_S; init_st.score = 0;
    beam.push_back(init_st);

    State best_completed;
    bool found_completed = false;

    // Phase 1 が終わるまで最大100ターン探索
    for (int turn = 0; turn < 100; ++turn) {
        if (beam.empty()) break;
        vector<State> next_beam;

        for (const State& st : beam) {
            bool all_empty = true;
            struct MoveInfo { int dest, k, step_val; };
            vector<MoveInfo> top_moves(R, {-1, 0, 0});
            
            for (int i = 0; i < R; ++i) {
                if (!st.D[i].empty()) {
                    all_empty = false;
                    int dest = st.D[i].back() / 10;
                    int k = 0;
                    for (int x = (int)st.D[i].size() - 1; x >= 0; --x) {
                        if (st.D[i][x] / 10 == dest) k++; else break;
                    }
                    top_moves[i] = {dest, k, st.D[i].back() % 10};
                }
            }
            
            // 全ての出発線が空になったら完了リスト（最強候補）へ
            if (all_empty) {
                if (!found_completed || st.score > best_completed.score) {
                    best_completed = st;
                    found_completed = true;
                }
                continue;
            }

            vector<pair<int, int>> mask_scores;
            for (int mask = 1; mask < (1 << R); ++mask) {
                int prev_dest = -1;
                bool valid = true;
                int local_score = 0;
                for (int i = 0; i < R; ++i) {
                    if ((mask >> i) & 1) {
                        if (top_moves[i].dest == -1) { valid = false; break; }
                        if (top_moves[i].dest <= prev_dest) { valid = false; break; }
                        prev_dest = top_moves[i].dest;
                        local_score += 1000 + top_moves[i].k * 10 + top_moves[i].step_val;
                    }
                }
                if (valid) mask_scores.push_back({local_score, mask});
            }

            // 局所的に優秀な上位 M 個の行動だけを展開
            sort(mask_scores.rbegin(), mask_scores.rend());
            int M = min(MAX_EXPAND, (int)mask_scores.size());
            for (int m = 0; m < M; ++m) {
                int mask = mask_scores[m].second;
                State nxt = st;
                for (int i = 0; i < R; ++i) {
                    if ((mask >> i) & 1) {
                        int dest = top_moves[i].dest;
                        int k = top_moves[i].k;
                        for (int _k = 0; _k < k; ++_k) {
                            nxt.S[dest].push_front(nxt.D[i].back());
                            nxt.D[i].pop_back();
                        }
                        nxt.seq_ops.push_back({0, i, dest, k});
                    }
                }
                // 評価値に少しノイズを入れて多様性を持たせる
                nxt.score = calc_eval(R, nxt.S, nxt.seq_ops.size()) + (xor32(rng) % 500);
                next_beam.push_back(nxt);
            }
        }

        if (next_beam.empty()) break;
        sort(next_beam.begin(), next_beam.end());
        if (next_beam.size() > BEAM_WIDTH) next_beam.resize(BEAM_WIDTH);
        beam = next_beam;
    }

    return best_completed;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    Timer timer;
    double TIME_LIMIT = 1.90;

    int R;
    if (!(cin >> R)) return 0;

    vector<deque<int>> initial_D(R);
    vector<deque<int>> initial_S(R);

    for (int i = 0; i < R; ++i) {
        for (int j = 0; j < 10; ++j) {
            int v;
            cin >> v;
            initial_D[i].push_back(v);
        }
    }

    uint32_t rng = 123456789;
    vector<vector<Op>> best_turns;
    int min_turns_count = 1000000;
    int loop_count = 0;

    // 時間が許す限り「ランダム化ビームサーチ」を回し続ける
    while (timer.elapsed() < TIME_LIMIT) {
        State phase1_res = run_beam_search(R, initial_D, initial_S, rng);
        loop_count++;

        vector<Op> seq_ops = phase1_res.seq_ops;
        vector<deque<int>> D = phase1_res.D;
        vector<deque<int>> S = phase1_res.S;

        // ==========================================
        // Phase 2A: 偶数列の怠惰な局所ソート
        // ==========================================
        for (int step = 0; step < 10; ++step) {
            for (int j = 0; j < R; j += 2) {
                int target_v = 10 * j + step;
                int s_depth = -1;
                for (int d = 0; d < (int)S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
                int d_depth = -1, d_size = D[j+1].size();
                for (int d = 0; d < d_size; ++d) if (D[j+1][d_size - 1 - d] == target_v) { d_depth = d; break; }

                if (s_depth != -1) {
                    int k = s_depth;
                    if (k > 0) {
                        for (int _k = 0; _k < k; ++_k) {
                            int v = S[j].front(); S[j].pop_front(); D[j+1].push_back(v);
                        }
                        seq_ops.push_back({1, j+1, j, k});
                    }
                    int v = S[j].front(); S[j].pop_front();
                    D[j].push_back(v);
                    seq_ops.push_back({1, j, j, 1});
                } else if (d_depth != -1) {
                    int k = d_depth;
                    for (int _k = 0; _k < k + 1; ++_k) {
                        int v = D[j+1].back(); D[j+1].pop_back(); S[j].push_front(v);
                    }
                    seq_ops.push_back({0, j+1, j, k + 1});
                    int target = S[j].front(); S[j].pop_front();
                    D[j].push_back(target);
                    seq_ops.push_back({1, j, j, 1});
                }
            }
        }

        // ==========================================
        // Phase 2B: 奇数列の怠惰な局所ソート
        // ==========================================
        for (int step = 0; step < 10; ++step) {
            for (int j = 1; j < R; j += 2) {
                int target_v = 10 * j + step;
                int s_depth = -1;
                for (int d = 0; d < (int)S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
                int buf_depth = -1;
                for (int d = 0; d < (int)S[j-1].size(); ++d) if (S[j-1][d] == target_v) { buf_depth = d; break; }

                if (s_depth != -1) {
                    int k = s_depth;
                    if (k > 0) {
                        for (int _k = 0; _k < k; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                        seq_ops.push_back({1, j, j, k});
                        for (int _k = 0; _k < k; ++_k) { int v = D[j].back(); D[j].pop_back(); S[j-1].push_front(v); }
                        seq_ops.push_back({0, j, j-1, k});
                    }
                    int v = S[j].front(); S[j].pop_front();
                    D[j].push_back(v);
                    seq_ops.push_back({1, j, j, 1});
                } else if (buf_depth != -1) {
                    int k = buf_depth;
                    if (k > 0) {
                        for (int _k = 0; _k < k; ++_k) { int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v); }
                        seq_ops.push_back({1, j, j-1, k});
                        for (int _k = 0; _k < k; ++_k) { int v = D[j].back(); D[j].pop_back(); S[j].push_front(v); }
                        seq_ops.push_back({0, j, j, k});
                    }
                    int v = S[j-1].front(); S[j-1].pop_front();
                    D[j].push_back(v);
                    seq_ops.push_back({1, j, j-1, 1});
                }
            }
        }

        // ==========================================
        // Phase 3: 並列化（スケジュールの圧縮）
        // ==========================================
        vector<vector<Op>> parallel_turns;
        vector<int> last_used_D(R, -1);
        vector<int> last_used_S(R, -1);

        for (const Op& op : seq_ops) {
            int min_turn = max(last_used_D[op.i], last_used_S[op.j]) + 1;
            int t = min_turn;
            while (true) {
                if (t >= (int)parallel_turns.size()) {
                    parallel_turns.push_back({op});
                    last_used_D[op.i] = t; last_used_S[op.j] = t; break;
                }
                bool cross = false;
                for (const Op& existing : parallel_turns[t]) {
                    if (op.i == existing.i || op.j == existing.j) { cross = true; break; }
                    if ((op.i - existing.i) * (op.j - existing.j) < 0) { cross = true; break; }
                }
                if (!cross) {
                    parallel_turns[t].push_back(op);
                    last_used_D[op.i] = t; last_used_S[op.j] = t; break;
                }
                t++;
            }
        }

        if (parallel_turns.size() < min_turns_count) {
            min_turns_count = parallel_turns.size();
            best_turns = parallel_turns;
        }
    }

    // 最良の結果を出力
    cout << best_turns.size() << "\n";
    for (const auto& turn : best_turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) {
            cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
        }
    }

    // 自分のPCでの確認用
    cerr << "Beam Search Runs: " << loop_count << "\n";
    cerr << "Best Turns: " << min_turns_count << "\n";

    return 0;
}