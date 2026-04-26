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
    // Phase 1: 完全ルーティング (D -> 目的のS)
    // ==========================================
    for (int i = 0; i < R; ++i) {
        while (!D[i].empty()) {
            int dest = D[i].back() / 10;
            int k = 0;
            for (int x = (int)D[i].size() - 1; x >= 0; --x) {
                if (D[i][x] / 10 == dest) k++;
                else break;
            }
            for (int _k = 0; _k < k; ++_k) {
                int v = D[i].back();
                D[i].pop_back();
                S[dest].push_front(v);
            }
            seq_ops.push_back({0, i, dest, k});
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
            
            int d_depth = -1; // D[j+1]の"末尾(操作口)"からの深さ
            int d_size = D[j+1].size();
            for (int d = 0; d < d_size; ++d) {
                if (D[j+1][d_size - 1 - d] == target_v) { d_depth = d; break; }
            }

            if (s_depth != -1) {
                // ターゲットが S[j] にある場合
                int k = s_depth;
                if (k > 0) {
                    // 上のゴミを D[j+1] へ退避 (戻さない！)
                    for (int _k = 0; _k < k; ++_k) {
                        int v = S[j].front(); S[j].pop_front();
                        D[j+1].push_back(v);
                    }
                    seq_ops.push_back({1, j+1, j, k});
                }
                // ターゲットを配置
                int v = S[j].front(); S[j].pop_front();
                D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
                
            } else if (d_depth != -1) {
                // ターゲットがバッファ D[j+1] にある場合
                int k = d_depth;
                // [小技] 邪魔なゴミk両と、ターゲット1両を「まとめて」S[j]に動かすと手数が1減る！
                for (int _k = 0; _k < k + 1; ++_k) {
                    int v = D[j+1].back(); D[j+1].pop_back();
                    S[j].push_front(v);
                }
                seq_ops.push_back({0, j+1, j, k + 1});
                
                // ターゲットは綺麗にS[j]の先頭に来ているので配置
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
            
            int buf_depth = -1; // S[j-1]の先頭からの深さ
            for (int d = 0; d < (int)S[j-1].size(); ++d) {
                if (S[j-1][d] == target_v) { buf_depth = d; break; }
            }

            if (s_depth != -1) {
                // ターゲットが S[j] にある場合
                int k = s_depth;
                if (k > 0) {
                    // 上のゴミを S[j-1] へ退避 (D[j]経由)
                    for (int _k = 0; _k < k; ++_k) {
                        int v = S[j].front(); S[j].pop_front(); D[j].push_back(v);
                    }
                    seq_ops.push_back({1, j, j, k});
                    
                    for (int _k = 0; _k < k; ++_k) {
                        int v = D[j].back(); D[j].pop_back(); S[j-1].push_front(v);
                    }
                    seq_ops.push_back({0, j, j-1, k});
                }
                // ターゲットを配置
                int v = S[j].front(); S[j].pop_front();
                D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
                
            } else if (buf_depth != -1) {
                // ターゲットがバッファ S[j-1] にある場合
                int k = buf_depth;
                if (k > 0) {
                    // 上のゴミを S[j] へ退避 (D[j]経由)
                    for (int _k = 0; _k < k; ++_k) {
                        int v = S[j-1].front(); S[j-1].pop_front(); D[j].push_back(v);
                    }
                    seq_ops.push_back({1, j, j-1, k});
                    
                    for (int _k = 0; _k < k; ++_k) {
                        int v = D[j].back(); D[j].pop_back(); S[j].push_front(v);
                    }
                    seq_ops.push_back({0, j, j, k});
                }
                // ターゲットを配置 (S[j-1] -> D[j] は直接可能)
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

    cout << parallel_turns.size() << "\n";
    for (const auto& turn : parallel_turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) {
            cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
        }
    }

    return 0;
}