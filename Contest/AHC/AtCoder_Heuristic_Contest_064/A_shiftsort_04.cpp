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
    // Phase 2A: 偶数列 (0, 2, 4, 6, 8) の局所ソート
    // バッファとして右隣の空き出発線 D[j+1] を使用
    // ==========================================
    for (int step = 0; step < 10; ++step) {
        for (int j = 0; j < R; j += 2) {
            int target_v = 10 * j + step;
            int depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) {
                if (S[j][d] == target_v) { depth = d; break; }
            }
            int k = depth;

            if (k > 0) {
                // 邪魔な車両を D[j+1] に退避
                for (int _k = 0; _k < k; ++_k) {
                    int v = S[j].front(); S[j].pop_front();
                    D[j+1].push_back(v);
                }
                seq_ops.push_back({1, j+1, j, k});
            }

            // ターゲットを D[j] に配置
            int v = S[j].front(); S[j].pop_front();
            D[j].push_back(v);
            seq_ops.push_back({1, j, j, 1});

            if (k > 0) {
                // 退避した車両を D[j+1] から S[j] に戻す
                for (int _k = 0; _k < k; ++_k) {
                    int pop_v = D[j+1].back(); D[j+1].pop_back();
                    S[j].push_front(pop_v);
                }
                seq_ops.push_back({0, j+1, j, k});
            }
        }
    }

    // ==========================================
    // Phase 2B: 奇数列 (1, 3, 5, 7, 9) の局所ソート
    // バッファとして左隣の空き待避線 S[j-1] を使用
    // ==========================================
    for (int step = 0; step < 10; ++step) {
        for (int j = 1; j < R; j += 2) {
            int target_v = 10 * j + step;
            int depth = -1;
            for (int d = 0; d < (int)S[j].size(); ++d) {
                if (S[j][d] == target_v) { depth = d; break; }
            }
            int k = depth;

            if (k > 0) {
                // 邪魔な車両を S[j-1] に退避 (直接S->Sはできないので、一旦D[j]を経由する)
                for (int _k = 0; _k < k; ++_k) {
                    int v = S[j].front(); S[j].pop_front();
                    D[j].push_back(v);
                }
                seq_ops.push_back({1, j, j, k});

                for (int _k = 0; _k < k; ++_k) {
                    int pop_v = D[j].back(); D[j].pop_back();
                    S[j-1].push_front(pop_v);
                }
                seq_ops.push_back({0, j, j-1, k});
            }

            // ターゲットを D[j] に配置
            int v = S[j].front(); S[j].pop_front();
            D[j].push_back(v);
            seq_ops.push_back({1, j, j, 1});

            if (k > 0) {
                // 退避した車両を S[j-1] から S[j] に戻す (再び D[j] を経由)
                for (int _k = 0; _k < k; ++_k) {
                    int pop_v = S[j-1].front(); S[j-1].pop_front();
                    D[j].push_back(pop_v);
                }
                seq_ops.push_back({1, j, j-1, k});

                for (int _k = 0; _k < k; ++_k) {
                    int pop_v = D[j].back(); D[j].pop_back();
                    S[j].push_front(pop_v);
                }
                seq_ops.push_back({0, j, j, k});
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