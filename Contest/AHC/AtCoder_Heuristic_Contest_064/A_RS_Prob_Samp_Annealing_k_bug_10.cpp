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

// 1回の試行を行う関数 (progress: 0.0 ~ 1.0 制限時間に対する進行度)
vector<vector<Op>> solve(int R, vector<deque<int>> D, vector<deque<int>> S, uint32_t& rng, double progress) {
    vector<Op> seq_ops;

    // ==========================================
    // Phase 1: 確率的サンプリング ＆ 温度管理付きバラ撒き
    // ==========================================
    while (true) {
        bool all_empty = true;
        struct MoveInfo { int dest, k, step_val; };
        vector<MoveInfo> top_moves(R, {-1, 0, 0});
        
        for (int i = 0; i < R; ++i) {
            if (!D[i].empty()) {
                all_empty = false;
                int dest = D[i].back() / 10;
                int k = 0;
                for (int x = (int)D[i].size() - 1; x >= 0; --x) {
                    if (D[i][x] / 10 == dest) k++;
                    else break;
                }
                int step_val = D[i].back() % 10;
                top_moves[i] = {dest, k, step_val};
            }
        }
        
        if (all_empty) break;

        vector<pair<int, int>> valid_masks; // {score, mask}
        for (int mask = 1; mask < (1 << R); ++mask) {
            int prev_dest = -1;
            int score = 0;
            bool valid = true;
            
            for (int i = 0; i < R; ++i) {
                if ((mask >> i) & 1) {
                    if (top_moves[i].dest == -1) { valid = false; break; }
                    if (top_moves[i].dest <= prev_dest) { valid = false; break; }
                    prev_dest = top_moves[i].dest;
                    
                    score += 10000 + top_moves[i].step_val * 100 + top_moves[i].k * 10;
                }
            }
            if (valid) valid_masks.push_back({score, mask});
        }

        int best_mask = 0;
        if (!valid_masks.empty()) {
            // スコアが高い順にソート
            sort(valid_masks.rbegin(), valid_masks.rend());
            
            // 温度管理: 進行度0の時は候補全体から選べるが、進行度1の時はトップ1しか選ばない
            int M = valid_masks.size();
            int max_idx = 1 + (int)((M - 1) * (1.0 - progress));
            
            // ルーレット選択: ランダムを掛け合わせることで、上位(0に近いインデックス)が高確率で選ばれる
            double r1 = (double)(xor32(rng) % 10000) / 10000.0;
            double r2 = (double)(xor32(rng) % 10000) / 10000.0;
            int idx = (int)(max_idx * r1 * r2);
            
            if (idx >= valid_masks.size()) idx = valid_masks.size() - 1;
            if (idx < 0) idx = 0;
            
            best_mask = valid_masks[idx].second;
        }

        // 選ばれた組み合わせを実行する
        for (int i = 0; i < R; ++i) {
            if ((best_mask >> i) & 1) {
                int dest = top_moves[i].dest;
                int k = top_moves[i].k;
                for (int _k = 0; _k < k; ++_k) {
                    int v = D[i].back();
                    D[i].pop_back();
                    S[dest].push_front(v);
                }
                seq_ops.push_back({0, i, dest, k});
            }
        }
    }

    // ==========================================
    // Phase 2A: 偶数列 (0, 2, 4, 6, 8) 怠惰局所ソート ＆ 連続ブロック最適化
    // ==========================================
    for (int j = 0; j < R; j += 2) {
        for (int step = 0; step < 10; ) {
            int target_v = 10 * j + step;
            
            int s_depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
            
            int d_depth = -1, d_size = D[j+1].size();
            for (int d = 0; d < d_size; ++d) if (D[j+1][d_size - 1 - d] == target_v) { d_depth = d; break; }

            int consecutive = 1;

            if (s_depth != -1) {
                if (s_depth > 0) {
                    for (int _k = 0; _k < s_depth; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j+1].push_back(v); }
                    seq_ops.push_back({1, j+1, j, s_depth});
                }
                // 【連続ブロック最適化】 次のターゲットも連続してSの先頭にあるならまとめて送る
                while (consecutive + step < 10 && S[j].size() > consecutive && S[j][consecutive] == target_v + consecutive) {
                    consecutive++;
                }
                for (int _k = 0; _k < consecutive; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                seq_ops.push_back({1, j, j, consecutive});
                
            } else if (d_depth != -1) {
                while (consecutive + step < 10 && d_depth + consecutive < d_size && D[j+1][d_size - 1 - (d_depth + consecutive)] == target_v + consecutive) {
                    consecutive++;
                }
                // ゴミとターゲットブロックをまとめてSに戻す
                int move_k = d_depth + consecutive;
                for (int _k = 0; _k < move_k; ++_k) { int v = D[j+1].back(); D[j+1].pop_back(); S[j].push_front(v); }
                seq_ops.push_back({0, j+1, j, move_k});
                
                // ターゲットブロックを配置
                for (int _k = 0; _k < consecutive; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                seq_ops.push_back({1, j, j, consecutive});
            }
            step += consecutive; // まとめて動かした分だけステップを進める
        }
    }

    // ==========================================
    // Phase 2B: 奇数列 (1, 3, 5, 7, 9) 怠惰局所ソート ＆ 連続ブロック最適化
    // ==========================================
    for (int j = 1; j < R; j += 2) {
        for (int step = 0; step < 10; ) {
            int target_v = 10 * j + step;
            
            int s_depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) if (S[j][d] == target_v) { s_depth = d; break; }
            
            int buf_depth = -1;
            for (int d = 0; d < (int)S[j-1].size(); ++d) if (S[j-1][d] == target_v) { buf_depth = d; break; }

            int consecutive = 1;

            if (s_depth != -1) {
                if (s_depth > 0) {
                    for (int _k = 0; _k < s_depth; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                    seq_ops.push_back({1, j, j, s_depth});
                    for (int _k = 0; _k < s_depth; ++_k) { int v = D[j].back(); D[j].pop_back(); S[j-1].push_front(v); }
                    seq_ops.push_back({0, j, j-1, s_depth});
                }
                while (consecutive + step < 10 && S[j].size() > consecutive && S[j][consecutive] == target_v + consecutive) consecutive++;
                
                for (int _k = 0; _k < consecutive; ++_k) { int v = S[j].front(); S[j].pop_front(); D[j].push_back(v); }
                seq_ops.push_back({1, j, j, consecutive});
                
            } else if (buf_depth != -1) {
                if (buf_depth > 0) {
                    for (int _k = 0; _k < buf_depth; ++_k) { int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v); }
                    seq_ops.push_back({1, j, j-1, buf_depth});
                    for (int _k = 0; _k < buf_depth; ++_k) { int v = D[j].back(); D[j].pop_back(); S[j].push_front(v); }
                    seq_ops.push_back({0, j, j, buf_depth});
                }
                while (consecutive + step < 10 && S[j-1].size() > consecutive && S[j-1][consecutive] == target_v + consecutive) consecutive++;
                
                for (int _k = 0; _k < consecutive; ++_k) { int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v); }
                seq_ops.push_back({1, j, j-1, consecutive});
            }
            step += consecutive;
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
    return parallel_turns;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    Timer timer;
    double TIME_LIMIT = 1.90; // AtCoderの実行時間制限2.0sに対するマージン

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

    // 時間が許す限りガチャを回し続ける
    while (true) {
        double elapsed = timer.elapsed();
        if (elapsed >= TIME_LIMIT) break;
        double progress = elapsed / TIME_LIMIT;

        vector<vector<Op>> current_turns = solve(R, initial_D, initial_S, rng, progress);
        loop_count++;

        if (current_turns.size() < min_turns_count) {
            min_turns_count = current_turns.size();
            best_turns = current_turns;
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
    cerr << "Best Turns: " << min_turns_count << "\n";

    return 0;
}