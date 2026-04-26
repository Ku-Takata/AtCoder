#include <iostream>
#include <vector>
#include <deque>
#include <algorithm>

using namespace std;

struct Op {
    int type, i, j, k;
};

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    int R;
    if (!(cin >> R)) return 0;

    vector<deque<int>> D(R);
    vector<deque<int>> S(R);

    for (int i = 0; i < R; ++i) {
        for (int j = 0; j < 10; ++j) {
            int v;
            cin >> v;
            D[i].push_back(v);
        }
    }

    vector<Op> seq_ops;

    // ==========================================
    // Phase 1: 並列化を極限まで高めた初期バラ撒き
    // ==========================================
    while (true) {
        bool all_empty = true;
        // top_moves[i] = {目的地の待避線, まとめて動かせる両数 k}
        vector<pair<int, int>> top_moves(R, {-1, 0});
        
        for (int i = 0; i < R; ++i) {
            if (!D[i].empty()) {
                all_empty = false;
                int dest = D[i].back() / 10;
                int k = 0;
                for (int x = (int)D[i].size() - 1; x >= 0; --x) {
                    if (D[i][x] / 10 == dest) k++;
                    else break;
                }
                top_moves[i] = {dest, k};
            }
        }
        
        if (all_empty) break;

        int best_mask = 0;
        int max_score = -1;

        // 2^R (1024) 通りの組み合わせから、交差しない最大の同時移動を探す
        for (int mask = 1; mask < (1 << R); ++mask) {
            int prev_dest = -1;
            int score = 0;
            bool valid = true;
            
            for (int i = 0; i < R; ++i) {
                if ((mask >> i) & 1) {
                    if (top_moves[i].first == -1) {
                        valid = false; break; // 空の線路は選べない
                    }
                    // 交差判定: iは昇順に処理しているため、destも単調増加でなければ交差する
                    if (top_moves[i].first <= prev_dest) {
                        valid = false; break; 
                    }
                    prev_dest = top_moves[i].first;
                    // スコア: 「同時操作数」を最優先しつつ、「移動させる両数k」をタイブレーカーにする
                    score += 1000 + top_moves[i].second; 
                }
            }
            
            if (valid && score > max_score) {
                max_score = score;
                best_mask = mask;
            }
        }

        // 選ばれた最適な組み合わせ(best_mask)の移動を実行する
        for (int i = 0; i < R; ++i) {
            if ((best_mask >> i) & 1) {
                int dest = top_moves[i].first;
                int k = top_moves[i].second;
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
    // Phase 2A: 偶数列 (0, 2, 4, 6, 8) の怠惰な局所ソート
    // ==========================================
    for (int step = 0; step < 10; ++step) {
        for (int j = 0; j < R; j += 2) {
            int target_v = 10 * j + step;
            
            int s_depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) {
                if (S[j][d] == target_v) { s_depth = d; break; }
            }
            
            int d_depth = -1;
            int d_size = D[j+1].size();
            for (int d = 0; d < d_size; ++d) {
                if (D[j+1][d_size - 1 - d] == target_v) { d_depth = d; break; }
            }

            if (s_depth != -1) {
                int k = s_depth;
                if (k > 0) {
                    for (int _k = 0; _k < k; ++_k) {
                        int v = S[j].front(); S[j].pop_front();
                        D[j+1].push_back(v);
                    }
                    seq_ops.push_back({1, j+1, j, k});
                }
                int v = S[j].front(); S[j].pop_front();
                D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
                
            } else if (d_depth != -1) {
                int k = d_depth;
                for (int _k = 0; _k < k + 1; ++_k) {
                    int v = D[j+1].back(); D[j+1].pop_back();
                    S[j].push_front(v);
                }
                seq_ops.push_back({0, j+1, j, k + 1});
                
                int target = S[j].front(); S[j].pop_front();
                D[j].push_back(target);
                seq_ops.push_back({1, j, j, 1});
            }
        }
    }

    // ==========================================
    // Phase 2B: 奇数列 (1, 3, 5, 7, 9) の怠惰な局所ソート
    // ==========================================
    for (int step = 0; step < 10; ++step) {
        for (int j = 1; j < R; j += 2) {
            int target_v = 10 * j + step;
            
            int s_depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) {
                if (S[j][d] == target_v) { s_depth = d; break; }
            }
            
            int buf_depth = -1;
            for (int d = 0; d < (int)S[j-1].size(); ++d) {
                if (S[j-1][d] == target_v) { buf_depth = d; break; }
            }

            if (s_depth != -1) {
                int k = s_depth;
                if (k > 0) {
                    for (int _k = 0; _k < k; ++_k) {
                        int v = S[j].front(); S[j].pop_front(); D[j].push_back(v);
                    }
                    seq_ops.push_back({1, j, j, k});
                    
                    for (int _k = 0; _k < k; ++_k) {
                        int v = D[j].back(); D[j].pop_back(); S[j-1].push_front(v);
                    }
                    seq_ops.push_back({0, j, j-1, k});
                }
                int v = S[j].front(); S[j].pop_front();
                D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
                
            } else if (buf_depth != -1) {
                int k = buf_depth;
                if (k > 0) {
                    for (int _k = 0; _k < k; ++_k) {
                        int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v);
                    }
                    seq_ops.push_back({1, j, j-1, k});
                    
                    for (int _k = 0; _k < k; ++_k) {
                        int v = D[j].back(); D[j].pop_back(); S[j].push_front(v);
                    }
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
                last_used_D[op.i] = t;
                last_used_S[op.j] = t;
                break;
            }

            bool cross = false;
            for (const Op& existing : parallel_turns[t]) {
                if (op.i == existing.i || op.j == existing.j) { cross = true; break; }
                if ((op.i - existing.i) * (op.j - existing.j) < 0) { cross = true; break; }
            }

            if (!cross) {
                parallel_turns[t].push_back(op);
                last_used_D[op.i] = t;
                last_used_S[op.j] = t;
                break;
            }
            t++;
        }
    }

    // 結果出力
    cout << parallel_turns.size() << "\n";
    for (const auto& turn : parallel_turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) {
            cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
        }
    }

    return 0;
}