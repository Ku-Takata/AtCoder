#include <iostream>
#include <vector>
#include <deque>
#include <algorithm>
#include <chrono>
#include <cmath>

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

double rand_double(uint32_t& rng) {
    return (double)(xor32(rng) % 100000) / 100000.0;
}

struct Op {
    int type, i, j, k;
};

// 重み配列 W を使ってシミュレーションを行う
vector<vector<Op>> solve(int R, vector<deque<int>> D, vector<deque<int>> S, const vector<int>& W) {
    vector<Op> seq_ops;

    // ==========================================
    // Phase 1: 重み配列(W)に基づく完全な貪欲バラ撒き
    // ==========================================
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
                    if (D[i][x] / 10 == dest) k++;
                    else break;
                }
                top_moves[i] = {dest, k, D[i].back()};
            }
        }
        
        if (all_empty) break;

        int best_mask = 0;
        int max_score = -2000000000;

        for (int mask = 1; mask < (1 << R); ++mask) {
            int prev_dest = -1;
            int score = 0;
            bool valid = true;
            
            for (int i = 0; i < R; ++i) {
                if ((mask >> i) & 1) {
                    if (top_moves[i].dest == -1) { valid = false; break; }
                    if (top_moves[i].dest <= prev_dest) { valid = false; break; }
                    prev_dest = top_moves[i].dest;
                    
                    score += 100000 + W[top_moves[i].v] + top_moves[i].k * 10;
                }
            }
            if (valid && score > max_score) {
                max_score = score;
                best_mask = mask;
            }
        }

        for (int i = 0; i < R; ++i) {
            if ((best_mask >> i) & 1) {
                int dest = top_moves[i].dest;
                int k = top_moves[i].k;
                for (int _k = 0; _k < k; ++_k) {
                    int v = D[i].back(); D[i].pop_back(); S[dest].push_front(v);
                }
                seq_ops.push_back({0, i, dest, k});
            }
        }
    }

    // ==========================================
    // Phase 2A: 偶数列 (0, 2, 4, 6, 8) の安全な怠惰ソート
    // ==========================================
    for (int step = 0; step < 10; ++step) {
        for (int j = 0; j < R; j += 2) {
            int target_v = 10 * j + step;
            int s_depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
            int d_depth = -1, d_size = D[j+1].size();
            for (int d = 0; d < d_size; ++d) if (D[j+1][d_size - 1 - d] == target_v) { d_depth = d; break; }

            if (s_depth != -1) {
                if (s_depth > 0) {
                    for (int _k = 0; _k < s_depth; ++_k) {
                        int v = S[j].front(); S[j].pop_front(); D[j+1].push_back(v);
                    }
                    seq_ops.push_back({1, j+1, j, s_depth}); // ブロック化はRLEに任せても良いが、ここは安全なブロック
                }
                int v = S[j].front(); S[j].pop_front(); D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
            } else if (d_depth != -1) {
                if (d_depth > 0) {
                    for (int _k = 0; _k < d_depth; ++_k) {
                        int v = D[j+1].back(); D[j+1].pop_back(); S[j].push_front(v);
                    }
                    seq_ops.push_back({0, j+1, j, d_depth});
                }
                int target = D[j+1].back(); D[j+1].pop_back(); S[j].push_front(target);
                seq_ops.push_back({0, j+1, j, 1});
                
                int v = S[j].front(); S[j].pop_front(); D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
            }
        }
    }

    // ==========================================
    // Phase 2B: 奇数列 (1, 3, 5, 7, 9) の安全な怠惰ソート
    // ==========================================
    for (int step = 0; step < 10; ++step) {
        for (int j = 1; j < R; j += 2) {
            int target_v = 10 * j + step;
            int s_depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
            int buf_depth = -1;
            for (int d = 0; d < (int)S[j-1].size(); ++d) if (S[j-1][d] == target_v) { buf_depth = d; break; }

            if (s_depth != -1) {
                if (s_depth > 0) {
                    for (int _k = 0; _k < s_depth; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                    seq_ops.push_back({1, j, j, s_depth});
                    for (int _k = 0; _k < s_depth; ++_k) { int v = D[j].back(); D[j].pop_back(); S[j-1].push_front(v); }
                    seq_ops.push_back({0, j, j-1, s_depth});
                }
                int v = S[j].front(); S[j].pop_front(); D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
            } else if (buf_depth != -1) {
                if (buf_depth > 0) {
                    for (int _k = 0; _k < buf_depth; ++_k) { int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v); }
                    seq_ops.push_back({1, j, j-1, buf_depth});
                    for (int _k = 0; _k < buf_depth; ++_k) { int v = D[j].back(); D[j].pop_back(); S[j].push_front(v); }
                    seq_ops.push_back({0, j, j, buf_depth});
                }
                int target = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(target);
                seq_ops.push_back({1, j, j-1, 1});
            }
        }
    }

    // ==========================================
    // RLE (ランレングス圧縮) による k両最適化の自動復元
    // ==========================================
    vector<Op> compressed_ops;
    for (const Op& op : seq_ops) {
        // 全く同じ移動経路・方向であれば、kを足し合わせて1手にする
        if (!compressed_ops.empty() && 
            compressed_ops.back().type == op.type && 
            compressed_ops.back().i == op.i && 
            compressed_ops.back().j == op.j) {
            compressed_ops.back().k += op.k;
        } else {
            compressed_ops.push_back(op);
        }
    }
    seq_ops = compressed_ops;

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
    return parallel_turns;
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
    
    // 重み配列の初期化
    vector<int> current_W(100, 0);
    for (int v = 0; v < 100; ++v) {
        current_W[v] = (v % 10) * 100;
    }

    vector<vector<Op>> best_turns = solve(R, initial_D, initial_S, current_W);
    int current_score = best_turns.size();
    int best_score = current_score;

    int loop_count = 0;
    double start_temp = 5.0;
    double end_temp = 0.01;

    // 時間が許す限り焼きなます
    while (true) {
        double elapsed = timer.elapsed();
        if (elapsed >= TIME_LIMIT) break;
        double progress = elapsed / TIME_LIMIT;
        
        double temp = start_temp * pow(end_temp / start_temp, progress);

        vector<int> next_W = current_W;
        int change_count = 1 + (xor32(rng) % 3);
        for (int i = 0; i < change_count; ++i) {
            int target_v = xor32(rng) % 100;
            int delta = (xor32(rng) % 1001) - 500; 
            next_W[target_v] += delta;
        }

        vector<vector<Op>> next_turns = solve(R, initial_D, initial_S, next_W);
        int next_score = next_turns.size();
        loop_count++;

        int diff = current_score - next_score; 
        
        if (diff > 0 || exp(diff / temp) > rand_double(rng)) {
            current_score = next_score;
            current_W = next_W;
            
            if (current_score < best_score) {
                best_score = current_score;
                best_turns = next_turns;
            }
        }
    }

    // ベストな結果を出力
    cout << best_turns.size() << "\n";
    for (const auto& turn : best_turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) {
            cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
        }
    }

    cerr << "Loop Count: " << loop_count << "\n";
    cerr << "Best Turns: " << best_score << "\n";

    return 0;
}