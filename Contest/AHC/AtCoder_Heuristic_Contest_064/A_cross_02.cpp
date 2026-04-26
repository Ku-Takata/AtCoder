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
    // Phase 1: 分解 (まとめて移動 k >= 1 を活用)
    // ==========================================
    for (int i = 0; i < R; ++i) {
        while (!D[i].empty()) {
            int target_dest = D[i].back() / 10;
            int k = 0;
            
            // 末尾から見て、同じ行き先の車両が連続して何両あるか数える
            for (int idx = (int)D[i].size() - 1; idx >= 0; --idx) {
                if (D[i][idx] / 10 == target_dest) k++;
                else break;
            }
            
            // k両まとめて待避線へ移動
            for (int step = 0; step < k; ++step) {
                int v = D[i].back();
                D[i].pop_back();
                S[target_dest].push_front(v);
            }
            seq_ops.push_back({0, i, target_dest, k});
        }
    }

    // ==========================================
    // Phase 2: 再構築 (層ごとに構築 ＆ まとめて退避)
    // ==========================================
    // stepを外側にすることで、全線路を並行して構築し、Phase 3での並列化効率を最大化する
    for (int step = 0; step < 10; ++step) {
        for (int j = 0; j < R; ++j) {
            int target_v = 10 * j + step;
            
            // 待避線 j の中からターゲットの深さ（先頭から何番目か）を探す
            int depth = -1;
            for (int idx = 0; idx < S[j].size(); ++idx) {
                if (S[j][idx] == target_v) {
                    depth = idx;
                    break;
                }
            }
            
            if (depth == -1) continue; // 安全策

            // ターゲットの上に邪魔な車両がある場合
            if (depth > 0) {
                // 最も空き容量に余裕のある出発線をバッファに選ぶ
                int B = -1;
                int min_size = 100;
                for (int i = 0; i < R; ++i) {
                    if (i != j && D[i].size() + depth <= 15) {
                        if (D[i].size() < min_size) {
                            min_size = D[i].size();
                            B = i;
                        }
                    }
                }

                // [まとめて退避] S[j] の先頭から depth 両を取り出し、D[B] の末尾へ
                for (int d = 0; d < depth; ++d) {
                    int v = S[j].front();
                    S[j].pop_front();
                    D[B].push_back(v);
                }
                seq_ops.push_back({1, B, j, depth});
                
                // [ターゲット移動] S[j] の先頭(ターゲット)を D[j] へ
                int v = S[j].front();
                S[j].pop_front();
                D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});

                // [まとめて復帰] D[B] の末尾から depth 両を取り出し、S[j] の先頭へ戻す
                for (int d = 0; d < depth; ++d) {
                    int v = D[B].back();
                    D[B].pop_back();
                    S[j].push_front(v);
                }
                seq_ops.push_back({0, B, j, depth});
                
            } else {
                // ターゲットが最初から先頭にある場合はそのまま移動
                int v = S[j].front();
                S[j].pop_front();
                D[j].push_back(v);
                seq_ops.push_back({1, j, j, 1});
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
            if (t >= parallel_turns.size()) {
                parallel_turns.push_back({op});
                last_used_D[op.i] = t;
                last_used_S[op.j] = t;
                break;
            }

            bool cross = false;
            for (const Op& existing : parallel_turns[t]) {
                if (op.i == existing.i || op.j == existing.j) {
                    cross = true;
                    break;
                }
                if ((op.i - existing.i) * (op.j - existing.j) < 0) {
                    cross = true;
                    break;
                }
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

    // ==========================================
    // 結果の出力
    // ==========================================
    cout << parallel_turns.size() << "\n";
    for (const auto& turn : parallel_turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) {
            cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
        }
    }

    return 0;
}