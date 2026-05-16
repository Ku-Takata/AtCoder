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

// --- BFS用の追加データ構造 ---
vector<pair<int, int>> cell_belts[20][20]; // 各マスが属する {ベルト番号, インデックス} のリスト
int dist_to_exit[20][20];                  // (0, 10) までの最短距離
pair<int, int> best_move[20][20];          // そのマスにいる時、(0, 10)に近づくための最適な操作 {ベルト番号, 方向}

// ベルトコンベアの配置を初期化（横10個、縦10個）
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

// (0, 10)を起点に逆向きにBFSを行い、全マスからの最短手数を事前計算する
void init_bfs() {
    for (int r = 0; r < 20; ++r) {
        for (int c = 0; c < 20; ++c) {
            dist_to_exit[r][c] = 1e9; // 初期値は無限大
        }
    }
    
    // 各マスがどのベルトの何番目のインデックスかを記録
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

            // 操作 +1 で (r, c) に到達できる元のマス (idx - 1)
            int prev_idx1 = (idx - 1 + 40) % 40;
            int pr1 = belts[m][prev_idx1].first;
            int pc1 = belts[m][prev_idx1].second;
            if (dist_to_exit[pr1][pc1] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr1][pc1] = dist_to_exit[r][c] + 1;
                best_move[pr1][pc1] = {m, 1}; // pr1, pc1 にいる時は m を +1 回すのが最適
                q.push({pr1, pc1});
            }

            // 操作 -1 で (r, c) に到達できる元のマス (idx + 1)
            int prev_idx2 = (idx + 1) % 40;
            int pr2 = belts[m][prev_idx2].first;
            int pc2 = belts[m][prev_idx2].second;
            if (dist_to_exit[pr2][pc2] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr2][pc2] = dist_to_exit[r][c] + 1;
                best_move[pr2][pc2] = {m, -1}; // pr2, pc2 にいる時は m を -1 回すのが最適
                q.push({pr2, pc2});
            }
        }
    }
}

// 指定したベルトを1マス回転させ、盤面と箱の位置情報を更新する
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

    // 操作後の搬出判定
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
    init_bfs(); // ★ BFSによる最短経路の事前計算

    // 初期状態で箱0が (0, 10) にある場合の特別処理
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

        // ★ BFSの計算結果に従って、(0, 10)に到着するまで最適な操作を繰り返す
        while (current_target == target_id && pos[target_id] != make_pair(0, 10)) {
            int r = pos[target_id].first;
            int c = pos[target_id].second;
            
            // 現在地から (0, 10) へ向かうための最短の一手を取得
            pair<int, int> move = best_move[r][c];
            shift_belt(move.first, move.second);
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