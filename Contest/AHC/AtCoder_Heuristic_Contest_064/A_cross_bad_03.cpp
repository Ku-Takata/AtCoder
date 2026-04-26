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
    vector<int> placed_count(R, 0);

    // ==========================================
    // 優先順位付きステートマシンによる構築
    // ==========================================
    while (true) {
        bool all_done = true;
        for (int i = 0; i < R; ++i) {
            if (placed_count[i] < 10) {
                all_done = false;
                break;
            }
        }
        if (all_done) break;

        bool action_taken = false;

        // -----------------------------------------------------------
        // Rule 1 & 2: 全路線を監視し、即時配置 または 受け入れ準備 を行う
        // -----------------------------------------------------------
        for (int j = 0; j < R; ++j) {
            if (placed_count[j] == 10) continue;
            int target = 10 * j + placed_count[j];
            bool d_clean = ((int)D[j].size() == placed_count[j]);

            int s_idx = -1, d_idx = -1;
            // ターゲットが「一番上」に露出しているかチェック
            for (int i = 0; i < R; ++i) {
                if (!S[i].empty() && S[i].front() == target) s_idx = i;
                if (!D[i].empty() && D[i].back() == target) d_idx = i;
            }

            if (d_clean) {
                // 【Rule 1】 D[j]が綺麗
                if (s_idx != -1) {
                    // Sの先頭にある -> 即時配置！
                    int v = S[s_idx].front();
                    S[s_idx].pop_front();
                    D[j].push_back(v);
                    placed_count[j]++;
                    seq_ops.push_back({1, j, s_idx, 1});
                    action_taken = true; break;
                }
                if (d_idx != -1) {
                    // Dの末尾にある -> そのままDへは入れられないので、一旦Sへ逃がす（次ターンで入る）
                    int target_s = d_idx; 
                    if (S[target_s].size() == 20) {
                        for (int i=0; i<R; ++i) if (S[i].size() < 20) { target_s = i; break; }
                    }
                    int v = D[d_idx].back();
                    D[d_idx].pop_back();
                    S[target_s].push_front(v);
                    seq_ops.push_back({0, d_idx, target_s, 1});
                    action_taken = true; break;
                }
            } else {
                // 【Rule 2】 D[j]にゴミが乗っている
                if (s_idx != -1) {
                    // ターゲットはSの先頭に見えているのにD[j]が詰まっている -> D[j]のゴミをSへ退避
                    int target_s = D[j].back() / 10;
                    if (S[target_s].size() == 20) {
                        for (int i=0; i<R; ++i) if (S[i].size() < 20) { target_s = i; break; }
                    }
                    // k両まとめて退避 (ここも int キャストを明示的に入れて安全化)
                    int k = 0;
                    for (int x = (int)D[j].size() - 1; x >= placed_count[j]; --x) {
                        if (D[j][x] / 10 == target_s) k++; else break;
                    }
                    k = min(k, 20 - (int)S[target_s].size());
                    if (k <= 0) k = 1;
                    for (int _k=0; _k<k; ++_k) {
                        int v = D[j].back();
                        D[j].pop_back();
                        S[target_s].push_front(v);
                    }
                    seq_ops.push_back({0, j, target_s, k});
                    action_taken = true; break;
                }
            }
        }
        if (action_taken) continue;

        // -----------------------------------------------------------
        // Rule 3: どこにも即時アクションがない場合、「１つのターゲット」に絞って掘り出す
        // -----------------------------------------------------------
        for (int j = 0; j < R; ++j) {
            if (placed_count[j] == 10) continue;
            int target = 10 * j + placed_count[j];

            int s_idx = -1, s_depth = -1;
            int d_idx = -1, d_depth = -1;

            // ターゲットの現在地を捜索
            for (int i=0; i<R; ++i) {
                for (int d=0; d<(int)S[i].size(); ++d) {
                    if (S[i][d] == target) { s_idx = i; s_depth = d; break; }
                }
                if (s_idx != -1) break;
            }
            if (s_idx == -1) {
                for (int i=0; i<R; ++i) {
                    for (int d = placed_count[i]; d < (int)D[i].size(); ++d) {
                        if (D[i][d] == target) { d_idx = i; d_depth = (int)D[i].size() - 1 - d; break; }
                    }
                    if (d_idx != -1) break;
                }
            }

            if (s_idx != -1) {
                // Sの奥にある -> 上の車両をDのバッファへ
                int B = -1;
                for (int i=0; i<R; ++i) if(i != j && (int)D[i].size() > placed_count[i] && (int)D[i].size() < 15) { B = i; break; }
                if (B == -1) {
                    for (int i=0; i<R; ++i) if(i != j && (int)D[i].size() < 15) { B = i; break; }
                }
                int dest_d = S[s_idx].front() / 10;
                int k = 0;
                for (int d=0; d<s_depth; ++d) {
                    if (S[s_idx][d] / 10 == dest_d) k++; else break;
                }
                k = min(k, 15 - (int)D[B].size());
                if (k <= 0) k = 1;
                for (int _k=0; _k<k; ++_k) {
                    int v = S[s_idx].front();
                    S[s_idx].pop_front();
                    D[B].push_back(v);
                }
                seq_ops.push_back({1, B, s_idx, k});
                action_taken = true; break;
            } else if (d_idx != -1) {
                // Dの奥にある -> 上の車両をSへ
                int target_s = D[d_idx].back() / 10;
                if (S[target_s].size() == 20) {
                    for (int i=0; i<R; ++i) if((int)S[i].size() < 20) { target_s = i; break; }
                }
                int max_k = d_depth > 0 ? d_depth : 1;
                int k = 0;
                
                // 【バグ修正箇所】 (int) キャストを明示してアンダーフローを防止
                int bound = (int)D[d_idx].size() - max_k;
                for (int x = (int)D[d_idx].size() - 1; x >= bound; --x) {
                    if (D[d_idx][x] / 10 == target_s) k++; else break;
                }
                
                k = min(k, 20 - (int)S[target_s].size());
                if (k <= 0) k = 1;
                for (int _k=0; _k<k; ++_k) {
                    int v = D[d_idx].back();
                    D[d_idx].pop_back();
                    S[target_s].push_front(v);
                }
                seq_ops.push_back({0, d_idx, target_s, k});
                action_taken = true; break;
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