#include <iostream>
#include <vector>
#include <deque>

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

    vector<vector<Op>> turns;

    // Phase 1: 分解 (D[i]末尾 -> S[dest]先頭)
    for (int i = 0; i < R; ++i) {
        while (!D[i].empty()) {
            int v = D[i].back();
            D[i].pop_back();
            int dest = v / 10;
            S[dest].push_front(v);
            turns.push_back({{0, i, dest, 1}});
        }
    }

    // Phase 2: 再構築
    for (int j = 0; j < R; ++j) {
        for (int step = 0; step < 10; ++step) {
            int target_v = 10 * j + step;
            vector<int> buffer_history;

            while (true) {
                if (S[j].empty()) break; // 安全策

                int v = S[j].front();
                S[j].pop_front();

                if (v == target_v) {
                    D[j].push_back(v);
                    // Type 1: Siding j -> Departure j (i=j, j=j)
                    turns.push_back({{1, j, j, 1}});
                    break;
                } else {
                    int B = -1;
                    for (int i = 0; i < R; ++i) {
                        if (i != j && D[i].size() < 15) {
                            B = i;
                            break;
                        }
                    }
                    D[B].push_back(v);
                    buffer_history.push_back(B);
                    // Type 1: Siding j -> Departure B (i=B, j=j)
                    // 前回のミス：ここが {1, j, B, 1} になっていました
                    turns.push_back({{1, B, j, 1}});
                }
            }

            // バッファから戻す (D[B]末尾 -> S[j]先頭)
            while (!buffer_history.empty()) {
                int B = buffer_history.back();
                buffer_history.pop_back();

                int v = D[B].back();
                D[B].pop_back();

                S[j].push_front(v);
                // Type 0: Departure B -> Siding j (i=B, j=j)
                turns.push_back({{0, B, j, 1}});
            }
        }
    }

    cout << turns.size() << "\n";
    for (const auto& turn : turns) {
        cout << turn.size() << "\n";
        for (const auto& op : turn) {
            cout << op.type << " " << op.i << " " << op.j << " " << op.k << "\n";
        }
    }

    return 0;
}