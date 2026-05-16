// 各マスベルトコンべアは最大2個まで、全体でN**2個まで
// 最大ターンは10**5
// ベルトコンベアは循環している必要があり、1マスずつ上にある物あるいは空が移動する
// 出口マスの箱の値が最も小さい場合は取り除かれる
// 盤面の大きさは20*20で固定

#include <iostream>
#include <vector>

using namespace std;

int N = 20;
vector<vector<int>> grid(20, vector<int>(20));
vector<pair<int, int>> pos(400); // 各箱(0~399)の現在位置
vector<vector<pair<int, int>>> belts(20);
vector<pair<int, int>> ops; // 操作の記録 {ベルト番号, 方向}
int current_target = 0; // 次に搬出したい箱の番号

// ベルトコンベアの配置を初期化（横10個、縦10個）
void init_belts() {
    // 横ベルト m = 0..9 (行 2*i と 2*i+1 を往復)
    for (int i = 0; i < 10; ++i) {
        for (int c = 0; c < 20; ++c) belts[i].push_back({2 * i, c});
        for (int c = 19; c >= 0; --c) belts[i].push_back({2 * i + 1, c});
    }
    // 縦ベルト m = 10..19 (列 2*j と 2*j+1 を往復)
    for (int j = 0; j < 10; ++j) {
        for (int r = 0; r < 20; ++r) belts[10 + j].push_back({r, 2 * j + 1});
        for (int r = 19; r >= 0; --r) belts[10 + j].push_back({r, 2 * j});
    }
}

// 指定したベルトを1マス回転させ、盤面と箱の位置情報を更新する
void shift_belt(int m, int d) {
    vector<int> temp(40);
    // 現在のベルト上の箱を一時保存
    for (int i = 0; i < 40; ++i) {
        temp[i] = grid[belts[m][i].first][belts[m][i].second];
    }
    // dの方向（1 or -1）にシフトして書き戻す
    for (int i = 0; i < 40; ++i) {
        int new_idx = (i + d + 40) % 40;
        int box_id = temp[i];
        grid[belts[m][new_idx].first][belts[m][new_idx].second] = box_id;
        if (box_id != -1) {
            pos[box_id] = belts[m][new_idx];
        }
    }
    ops.push_back({m, d});

    // 操作後の搬出判定: (0, 10) に current_target が来たら消滅
    if (grid[0][10] == current_target) {
        grid[0][10] = -1;
        pos[current_target] = {-1, -1};
        current_target++;
    }
}

int main() {
    // 入出力の高速化
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

    // 初期状態で箱0が (0, 10) にある場合の特別処理
    if (grid[0][10] == 0) {
        grid[0][10] = -1;
        pos[0] = {-1, -1};
        current_target = 1;
    }

    // すべての箱を順番に搬出する
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
        int r = pos[target_id].first;
        int c = pos[target_id].second;

        // ステップ1: 横ベルトを使って、ターゲットを列10または11（縦ベルト15の担当領域）へ運ぶ
        if (c != 10 && c != 11) {
            int h_belt = r / 2;
            pair<int, int> target_cell = {2 * h_belt, 10}; 
            
            int curr_idx = -1, target_idx = -1;
            for (int i = 0; i < 40; ++i) {
                if (belts[h_belt][i] == pos[target_id]) curr_idx = i;
                if (belts[h_belt][i] == target_cell) target_idx = i;
            }
            
            int diff = (target_idx - curr_idx + 40) % 40;
            int d = (diff <= 20) ? 1 : -1;
            
            while (current_target == target_id && pos[target_id] != target_cell) {
                shift_belt(h_belt, d);
            }
        }

        // ステップ2: 縦ベルト15を使って、ターゲットを出口 (0, 10) へ運ぶ
        // ★修正点: ベルトインデックスを5から「15」に変更
        if (current_target == target_id && pos[target_id] != make_pair(0, 10)) {
            int curr_idx = -1, target_idx = -1;
            for (int i = 0; i < 40; ++i) {
                if (belts[15][i] == pos[target_id]) curr_idx = i;
                if (belts[15][i] == make_pair(0, 10)) target_idx = i;
            }
            
            int diff = (target_idx - curr_idx + 40) % 40;
            int d = (diff <= 20) ? 1 : -1;
            
            while (current_target == target_id && pos[target_id] != make_pair(0, 10)) {
                shift_belt(15, d);
            }
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