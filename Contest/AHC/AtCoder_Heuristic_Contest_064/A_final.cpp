#include <iostream>
#include <vector>
#include <deque>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <unordered_set>

using namespace std;

struct Timer {
    chrono::high_resolution_clock::time_point st;
    Timer() { st = chrono::high_resolution_clock::now(); }
    double elapsed() {
        return chrono::duration_cast<chrono::duration<double>>(chrono::high_resolution_clock::now() - st).count();
    }
};

uint32_t xor32(uint32_t& x) {
    x ^= x << 13; x ^= x >> 17; x ^= x << 5; return x;
}

double rand_double(uint32_t& rng) {
    return (double)(xor32(rng) % 100000) / 100000.0;
}

struct Op { int type, i, j, k; };

// Phase 2 用のビームサーチ状態（完全物理シミュレーション）
struct P2State {
    vector<int> pegs[3]; // 0: S[j], 1: D[j], 2: Buffer
    vector<pair<int, pair<int, int>>> ops; // from, to, k
    int score;
    bool operator<(const P2State& o) const { return score > o.score; }
};

uint64_t hash_p2(const P2State& st) {
    uint64_t h = 0;
    for (int i = 0; i < 3; ++i) {
        h ^= (i + 1);
        for (int x : st.pegs[i]) h = (h * 31) + x + 1;
        h *= 1000000007;
    }
    return h;
}

// --------------------------------------------------------
// Phase 2 の局所パズルをビームサーチで完全解明する関数
// --------------------------------------------------------
vector<Op> solve_track_beam_search(int j, const deque<int>& S_init) {
    P2State init_st;
    // S_initは先頭(0)が操作口なので、そのままpegs[0]に入れる
    for (int x : S_init) init_st.pegs[0].push_back(x % 10); 
    init_st.score = 0;

    // 各ペグの属性（true なら S: 先頭出し入れ, false なら D: 末尾出し入れ）
    bool is_S[3];
    is_S[0] = true;  // S[j]
    is_S[1] = false; // D[j]
    is_S[2] = (j % 2 != 0); // 偶数ならD[j+1](false), 奇数ならS[j-1](true)

    // 許可された移動ペア (SとDの間しか移動できない)
    vector<pair<int, int>> valid_pairs;
    if (j % 2 == 0) valid_pairs = {{0,1}, {1,0}, {0,2}, {2,0}}; 
    else            valid_pairs = {{0,1}, {1,0}, {2,1}, {1,2}}; 

    vector<P2State> beam = {init_st};
    unordered_set<uint64_t> visited;
    visited.insert(hash_p2(init_st));

    P2State best_st = init_st;
    bool found = false;

    // 最大100手まで探索
    for (int depth = 0; depth < 100; ++depth) {
        if (beam.empty() || found) break;
        vector<P2State> next_beam;

        for (const P2State& st : beam) {
            // ゴール判定: D[j](pegs[1]) に 0〜9 が正しく積まれているか
            if (st.pegs[1].size() == 10) {
                bool ok = true;
                for (int i = 0; i < 10; ++i) if (st.pegs[1][i] != i) ok = false;
                if (ok) { best_st = st; found = true; break; }
            }

            for (auto pair : valid_pairs) {
                int from = pair.first, to = pair.second;
                if (st.pegs[from].empty()) continue;

                // k両移動の全パターンを試す
                for (int k = 1; k <= st.pegs[from].size(); ++k) {
                    P2State nxt = st;
                    vector<int> chunk;
                    
                    // 【物理法則のシミュレーション】 Pop操作
                    if (is_S[from]) {
                        // Sは先頭から取る
                        chunk.assign(nxt.pegs[from].begin(), nxt.pegs[from].begin() + k);
                        nxt.pegs[from].erase(nxt.pegs[from].begin(), nxt.pegs[from].begin() + k);
                    } else {
                        // Dは末尾から取る
                        chunk.assign(nxt.pegs[from].end() - k, nxt.pegs[from].end());
                        nxt.pegs[from].erase(nxt.pegs[from].end() - k, nxt.pegs[from].end());
                    }

                    // 【物理法則のシミュレーション】 Push操作
                    if (is_S[to]) {
                        // Sは先頭に入れる
                        nxt.pegs[to].insert(nxt.pegs[to].begin(), chunk.begin(), chunk.end());
                    } else {
                        // Dは末尾に入れる
                        nxt.pegs[to].insert(nxt.pegs[to].end(), chunk.begin(), chunk.end());
                    }

                    uint64_t h = hash_p2(nxt);
                    if (visited.count(h)) continue;
                    visited.insert(h);

                    nxt.ops.push_back({from, {to, k}});
                    
                    // 評価関数
                    int correct = 0;
                    for (int i = 0; i < nxt.pegs[1].size(); ++i) {
                        if (nxt.pegs[1][i] == i) correct++; else break;
                    }
                    
                    // 欲しい数字が操作口にあるとボーナス
                    int s_bonus = 0;
                    if (!nxt.pegs[0].empty() && nxt.pegs[0].front() == correct) s_bonus += 50;
                    if (is_S[2] && !nxt.pegs[2].empty() && nxt.pegs[2].front() == correct) s_bonus += 50;
                    if (!is_S[2] && !nxt.pegs[2].empty() && nxt.pegs[2].back() == correct) s_bonus += 50;
                    
                    nxt.score = correct * 1000 - (int)nxt.ops.size() * 10 - nxt.pegs[2].size() * 5 + s_bonus;
                    next_beam.push_back(nxt);
                }
            }
            if (found) break;
        }

        if (found) break;
        sort(next_beam.begin(), next_beam.end());
        if (next_beam.size() > 150) next_beam.resize(150); // 時間内に終わるようにビーム幅を調整
        beam = next_beam;
    }

    // 抽象化された操作を実際の出力ルールに変換
    vector<Op> actual_ops;
    for (auto& o : best_st.ops) {
        int from = o.first, to = o.second.first, k = o.second.second;
        if (j % 2 == 0) {
            if (from==0 && to==1) actual_ops.push_back({1, j, j, k});
            if (from==1 && to==0) actual_ops.push_back({0, j, j, k});
            if (from==0 && to==2) actual_ops.push_back({1, j+1, j, k});
            if (from==2 && to==0) actual_ops.push_back({0, j+1, j, k});
        } else {
            if (from==0 && to==1) actual_ops.push_back({1, j, j, k});
            if (from==1 && to==0) actual_ops.push_back({0, j, j, k});
            if (from==2 && to==1) actual_ops.push_back({1, j, j-1, k});
            if (from==1 && to==2) actual_ops.push_back({0, j, j-1, k});
        }
    }
    return actual_ops;
}

// --------------------------------------------------------
// Phase 1: 重み配列(W)に基づく完全な貪欲バラ撒き
// --------------------------------------------------------
void solve_phase1(int R, vector<deque<int>> D, vector<deque<int>> S, const vector<int>& W, vector<Op>& seq_ops, vector<deque<int>>& final_S) {
    while (true) {
        bool all_empty = true;
        struct MoveInfo { int dest, k, v; };
        vector<MoveInfo> top_moves(R, {-1, 0, -1});
        
        for (int i = 0; i < R; ++i) {
            if (!D[i].empty()) {
                all_empty = false;
                int dest = D[i].back() / 10;
                int k = 0;
                for (int x = (int)D[i].size() - 1; x >= 0; --x) {
                    if (D[i][x] / 10 == dest) k++; else break;
                }
                top_moves[i] = {dest, k, D[i].back()};
            }
        }
        
        if (all_empty) break;
        int best_mask = 0;
        int max_score = -2000000000;

        for (int mask = 1; mask < (1 << R); ++mask) {
            int prev_dest = -1, score = 0;
            bool valid = true;
            for (int i = 0; i < R; ++i) {
                if ((mask >> i) & 1) {
                    if (top_moves[i].dest == -1 || top_moves[i].dest <= prev_dest) { valid = false; break; }
                    prev_dest = top_moves[i].dest;
                    score += 100000 + W[top_moves[i].v] + top_moves[i].k * 10;
                }
            }
            if (valid && score > max_score) { max_score = score; best_mask = mask; }
        }

        for (int i = 0; i < R; ++i) {
            if ((best_mask >> i) & 1) {
                int dest = top_moves[i].dest, k = top_moves[i].k;
                for (int _k = 0; _k < k; ++_k) {
                    int v = D[i].back(); D[i].pop_back(); S[dest].push_front(v);
                }
                seq_ops.push_back({0, i, dest, k});
            }
        }
    }
    final_S = S;
}

// --------------------------------------------------------
// SA評価用の軽量 Phase 2 (1両ずつ怠惰ソート)
// --------------------------------------------------------
int evaluate_phase2_fast(int R, vector<deque<int>> D, vector<deque<int>> S) {
    int turns = 0;
    for (int step = 0; step < 10; ++step) {
        for (int j = 0; j < R; j += 2) {
            int target_v = 10 * j + step;
            int s_depth = -1; for (int d = 0; d < S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
            int d_depth = -1, d_sz = D[j+1].size(); for (int d = 0; d < d_sz; ++d) if (D[j+1][d_sz - 1 - d] == target_v) { d_depth = d; break; }

            if (s_depth != -1) {
                turns += s_depth + 1;
                for(int k=0; k<s_depth; ++k) { int v = S[j].front(); S[j].pop_front(); D[j+1].push_back(v); }
                S[j].pop_front();
            } else if (d_depth != -1) {
                turns += d_depth + 2;
                for(int k=0; k<d_depth; ++k) { int v = D[j+1].back(); D[j+1].pop_back(); S[j].push_front(v); }
                D[j+1].pop_back();
            }
        }
    }
    for (int step = 0; step < 10; ++step) {
        for (int j = 1; j < R; j += 2) {
            int target_v = 10 * j + step;
            int s_depth = -1; for (int d = 0; d < S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
            int b_depth = -1; for (int d = 0; d < S[j-1].size(); ++d) if (S[j-1][d] == target_v) { b_depth = d; break; }

            if (s_depth != -1) {
                turns += s_depth * 2 + 1;
                for(int k=0; k<s_depth; ++k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                for(int k=0; k<s_depth; ++k) { int v = D[j].back(); D[j].pop_back(); S[j-1].push_front(v); }
                S[j].pop_front();
            } else if (b_depth != -1) {
                turns += b_depth * 2 + 1;
                for(int k=0; k<b_depth; ++k) { int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v); }
                for(int k=0; k<b_depth; ++k) { int v = D[j].back(); D[j].pop_back(); S[j].push_front(v); }
                S[j-1].pop_front();
            }
        }
    }
    return turns;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    Timer timer;
    double TIME_LIMIT = 1.80; // SAは1.80秒で切り上げ、残りの時間で重いビームサーチを回す

    int R;
    if (!(cin >> R)) return 0;

    vector<deque<int>> initial_D(R), initial_S(R);
    for (int i = 0; i < R; ++i) {
        for (int j = 0; j < 10; ++j) { int v; cin >> v; initial_D[i].push_back(v); }
    }

    uint32_t rng = 123456789;
    vector<int> current_W(100, 0);
    for (int v = 0; v < 100; ++v) current_W[v] = (v % 10) * 100;

    vector<Op> best_phase1_ops;
    vector<deque<int>> best_S_state;
    
    solve_phase1(R, initial_D, initial_S, current_W, best_phase1_ops, best_S_state);
    int current_score = best_phase1_ops.size() + evaluate_phase2_fast(R, initial_D, best_S_state);
    int best_score = current_score;

    int loop_count = 0;
    double start_temp = 5.0, end_temp = 0.01;

    // ==========================================
    // SA ループ
    // ==========================================
    while (true) {
        double elapsed = timer.elapsed();
        if (elapsed >= TIME_LIMIT) break;
        double progress = elapsed / TIME_LIMIT;
        double temp = start_temp * pow(end_temp / start_temp, progress);

        vector<int> next_W = current_W;
        int change_count = 1 + (xor32(rng) % 3);
        for (int i = 0; i < change_count; ++i) {
            next_W[xor32(rng) % 100] += (xor32(rng) % 1001) - 500;
        }

        vector<Op> temp_ops;
        vector<deque<int>> temp_S;
        solve_phase1(R, initial_D, initial_S, next_W, temp_ops, temp_S);
        int next_score = temp_ops.size() + evaluate_phase2_fast(R, initial_D, temp_S);
        loop_count++;

        int diff = current_score - next_score; 
        if (diff > 0 || exp(diff / temp) > rand_double(rng)) {
            current_score = next_score; current_W = next_W;
            if (current_score < best_score) {
                best_score = current_score;
                best_phase1_ops = temp_ops;
                best_S_state = temp_S;
            }
        }
    }

    // ==========================================
    // 最終結果に対してのみ Phase 2 ビームサーチを実行
    // ==========================================
    vector<Op> final_ops = best_phase1_ops;
    for (int j = 0; j < R; ++j) {
        vector<Op> p2_ops = solve_track_beam_search(j, best_S_state[j]);
        final_ops.insert(final_ops.end(), p2_ops.begin(), p2_ops.end());
    }

    // ==========================================
    // Phase 3: 並列化（スケジュールの圧縮）
    // ==========================================
    vector<vector<Op>> parallel_turns;
    vector<int> last_used_D(R, -1), last_used_S(R, -1);

    for (const Op& op : final_ops) {
        int t = max(last_used_D[op.i], last_used_S[op.j]) + 1;
        while (true) {
            if (t >= parallel_turns.size()) {
                parallel_turns.push_back({op});
                last_used_D[op.i] = t; last_used_S[op.j] = t; break;
            }
            bool cross = false;
            for (const Op& ex : parallel_turns[t]) {
                if (op.i == ex.i || op.j == ex.j || (op.i - ex.i) * (op.j - ex.j) < 0) { cross = true; break; }
            }
            if (!cross) {
                parallel_turns[t].push_back(op);
                last_used_D[op.i] = t; last_used_S[op.j] = t; break;
            }
            t++;
        }
    }

    cout << parallel_turns.size() << "\n";
    for (const auto& turn : parallel_turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
    }

    cerr << "SA Loop Count: " << loop_count << "\n";

    return 0;
}