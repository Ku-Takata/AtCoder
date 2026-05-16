#include <iostream>
#include <vector>
#include <queue>

using namespace std;

int N = 20;
vector<vector<int>> grid(20, vector<int>(20));
vector<pair<int, int>> pos(400); // 各箱(0~399)の現在位置
vector<vector<pair<int, int>>> belts(20);
vector<pair<int, int>> ops; // 操作の記録 {ベルト番号, 方向}
int current_target = 0; // 次に搬出したい箱の番号

// --- BFS/評価用のデータ構造 ---
vector<pair<int, int>> cell_belts[20][20]; // 各マスが属する {ベルト番号, インデックス} のリスト
int dist_to_exit[20][20];                  // (0, 10) までの最短距離

// ベルトコンベアの配置を初期化
void init_belts() {
    for (int i = 0; i < 10; ++i) {
        for (int c = 0; c < 20; ++c) belts[i].push_back({2 * i, c});
        for (int c = 19; c >= 0; --c) belts[i].push_back({2 * i + 1, c});
    }
    for (int j = 0; j < 10; ++j) {
        for (int r = 0; r < 20; ++r) belts[10 + j].push_back({r, 2 * j + 1});
        for (int r = 19; r >= 0; --r) belts[10 + j].push_back({r, 2 * j});
    }
}

// 全マスからの最短手数を事前計算
void init_bfs() {
    for (int r = 0; r < 20; ++r) {
        for (int c = 0; c < 20; ++c) {
            dist_to_exit[r][c] = 1e9;
        }
    }
    
    for (int m = 0; m < 20; ++m) {
        for (int i = 0; i < 40; ++i) {
            int r = belts[m][i].first;
            int c = belts[m][i].second;
            cell_belts[r][c].push_back({m, i});
        }
    }

    queue<pair<int, int>> q;
    q.push({0, 10});
    dist_to_exit[0][10] = 0;

    while (!q.empty()) {
        auto p = q.front();
        int r = p.first;
        int c = p.second;
        q.pop();

        for (auto b : cell_belts[r][c]) {
            int m = b.first;
            int idx = b.second;

            int prev_idx1 = (idx - 1 + 40) % 40;
            int pr1 = belts[m][prev_idx1].first;
            int pc1 = belts[m][prev_idx1].second;
            if (dist_to_exit[pr1][pc1] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr1][pc1] = dist_to_exit[r][c] + 1;
                q.push({pr1, pc1});
            }

            int prev_idx2 = (idx + 1) % 40;
            int pr2 = belts[m][prev_idx2].first;
            int pc2 = belts[m][prev_idx2].second;
            if (dist_to_exit[pr2][pc2] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr2][pc2] = dist_to_exit[r][c] + 1;
                q.push({pr2, pc2});
            }
        }
    }
}

void shift_belt(int m, int d) {
    vector<int> temp(40);
    for (int i = 0; i < 40; ++i) {
        temp[i] = grid[belts[m][i].first][belts[m][i].second];
    }
    for (int i = 0; i < 40; ++i) {
        int new_idx = (i + d + 40) % 40;
        int box_id = temp[i];
        grid[belts[m][new_idx].first][belts[m][new_idx].second] = box_id;
        if (box_id != -1) {
            pos[box_id] = belts[m][new_idx];
        }
    }
    ops.push_back({m, d});

    if (grid[0][10] == current_target) {
        grid[0][10] = -1;
        pos[current_target] = {-1, -1};
        current_target++;
    }
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    int dummy_N;
    if (!(cin >> dummy_N)) return 0;

    for (int i = 0; i < 20; ++i) {
        for (int j = 0; j < 20; ++j) {
            cin >> grid[i][j];
            pos[grid[i][j]] = {i, j};
        }
    }

    init_belts();
    init_bfs();

    if (grid[0][10] == 0) {
        grid[0][10] = -1;
        pos[0] = {-1, -1};
        current_target = 1;
    }

    while (current_target < 400) {
        if (pos[current_target].first == -1) {
            current_target++;
            continue;
        }

        if (pos[current_target] == make_pair(0, 10)) {
            shift_belt(1, 1);
            shift_belt(1, -1);
            continue;
        }

        int target_id = current_target;

        while (current_target == target_id && pos[target_id] != make_pair(0, 10)) {
            int r = pos[target_id].first;
            int c = pos[target_id].second;
            
            int best_m = -1;
            int best_d = 0;
            long long best_score = 2e18; // 最小化するので初期値は巨大に

            // 箱 target_id が乗っているベルト（最大2つ）のそれぞれについて、1方向と-1方向をシミュレート
            for (auto b : cell_belts[r][c]) {
                int m = b.first;
                for (int d : {1, -1}) {
                    long long current_score = 0;
                    // 先読みする箱の優先度ウェイト（k を絶対優先しつつ、k+1, k+2... をタイブレークに使う）
                    long long weights[] = {1000000000000LL, 100000000LL, 10000LL, 1LL}; 
                    int num_lookahead = 4; // k, k+1, k+2, k+3 の4箱分を評価

                    for (int offset = 0; offset < num_lookahead; ++offset) {
                        int check_id = target_id + offset;
                        if (check_id >= 400) break;
                        if (pos[check_id].first == -1) continue; // 既に搬出済みの箱は無視

                        pair<int, int> next_pos = pos[check_id];
                        
                        // check_id の箱が、今動かそうとしているベルト m 上にいるか判定
                        bool on_belt = false;
                        int current_idx = -1;
                        for (auto cb : cell_belts[next_pos.first][next_pos.second]) {
                            if (cb.first == m) {
                                on_belt = true;
                                current_idx = cb.second;
                                break;
                            }
                        }

                        // もしベルト m 上にいれば、d方向に動いた後の座標を計算
                        if (on_belt) {
                            int next_idx = (current_idx + d + 40) % 40;
                            next_pos = belts[m][next_idx];
                        }

                        // 評価値に加算: (その箱の出口までの最短距離) × (優先度ウェイト)
                        current_score += weights[offset] * dist_to_exit[next_pos.first][next_pos.second];
                    }

                    // 最も評価値が小さい（＝総合的に出口に近い）操作を記録
                    if (current_score < best_score) {
                        best_score = current_score;
                        best_m = m;
                        best_d = d;
                    }
                }
            }

            // 選ばれた最良の手を適用
            shift_belt(best_m, best_d);
        }
    }

    // --- 出力フェーズ ---
    cout << 20 << "\n";
    for (int i = 0; i < 20; ++i) {
        cout << 40;
        for (auto p : belts[i]) {
            cout << " " << p.first << " " << p.second;
        }
        cout << "\n";
    }
    
    cout << ops.size() << "\n";
    for (auto op : ops) {
        cout << op.first << " " << op.second << "\n";
    }

    return 0;
}