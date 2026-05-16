#include <iostream>
#include <queue>
#include <algorithm>
#include <chrono>

using namespace std;
using namespace std::chrono;

// --- 高速乱数生成器 (Xorshift) ---
uint32_t xor128() {
    static uint32_t x = 123456789, y = 362436069, z = 521288629, w = 88675123;
    uint32_t t = x ^ (x << 11);
    x = y; y = z; z = w;
    return w = (w ^ (w >> 19)) ^ (t ^ (t >> 8));
}

int init_grid[20][20];
int init_pos_r[400];
int init_pos_c[400];

int grid[20][20];
int pos_r[400];
int pos_c[400];

int belt_r[20][40];
int belt_c[20][40];

int cb_m[20][20][2];
int cb_idx[20][20][2];
int cb_cnt[20][20];

int dist_to_exit[20][20];

int ops_m[100005];
int ops_d[100005];
int ops_count = 0;

int current_target = 0;

// ベストスコア記録用
int best_ops_count = 1e9;
int best_ops_m[100005];
int best_ops_d[100005];

void init_belts() {
    for (int i = 0; i < 10; ++i) {
        for (int c = 0; c < 20; ++c) { belt_r[i][c] = 2 * i; belt_c[i][c] = c; }
        for (int c = 19; c >= 0; --c) { belt_r[i][39 - c] = 2 * i + 1; belt_c[i][39 - c] = c; }
    }
    for (int j = 0; j < 10; ++j) {
        for (int r = 0; r < 20; ++r) { belt_r[10 + j][r] = r; belt_c[10 + j][r] = 2 * j + 1; }
        for (int r = 19; r >= 0; --r) { belt_r[10 + j][39 - r] = r; belt_c[10 + j][39 - r] = 2 * j; }
    }
}

void init_bfs() {
    for (int r = 0; r < 20; ++r) {
        for (int c = 0; c < 20; ++c) {
            dist_to_exit[r][c] = 1e9;
            cb_cnt[r][c] = 0;
        }
    }
    for (int m = 0; m < 20; ++m) {
        for (int i = 0; i < 40; ++i) {
            int r = belt_r[m][i];
            int c = belt_c[m][i];
            int cnt = cb_cnt[r][c];
            cb_m[r][c][cnt] = m;
            cb_idx[r][c][cnt] = i;
            cb_cnt[r][c]++;
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
        for (int i = 0; i < cb_cnt[r][c]; ++i) {
            int m = cb_m[r][c][i];
            int idx = cb_idx[r][c][i];
            
            int prev_idx1 = (idx - 1 + 40) % 40;
            int pr1 = belt_r[m][prev_idx1];
            int pc1 = belt_c[m][prev_idx1];
            if (dist_to_exit[pr1][pc1] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr1][pc1] = dist_to_exit[r][c] + 1;
                q.push({pr1, pc1});
            }
            
            int prev_idx2 = (idx + 1) % 40;
            int pr2 = belt_r[m][prev_idx2];
            int pc2 = belt_c[m][prev_idx2];
            if (dist_to_exit[pr2][pc2] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr2][pc2] = dist_to_exit[r][c] + 1;
                q.push({pr2, pc2});
            }
        }
    }
}

// 毎回のシミュレーション開始前に盤面をリセットする
void reset_state() {
    for (int i = 0; i < 20; ++i) {
        for (int j = 0; j < 20; ++j) {
            grid[i][j] = init_grid[i][j];
        }
    }
    for (int i = 0; i < 400; ++i) {
        pos_r[i] = init_pos_r[i];
        pos_c[i] = init_pos_c[i];
    }
    ops_count = 0;
    current_target = 0;
    
    // 箱0が最初からゴールにいる場合の特別処理
    if (grid[0][10] == 0) {
        grid[0][10] = -1;
        pos_r[0] = -1;
        pos_c[0] = -1;
        current_target = 1;
    }
}

pair<int, int> get_best_move_strict(int target) {
    int r = pos_r[target];
    int c = pos_c[target];
    int current_dist = dist_to_exit[r][c];

    int best_m = -1;
    int best_d = 0;
    long long best_sub_score = 2e18;

    for (int i = 0; i < cb_cnt[r][c]; ++i) {
        int m = cb_m[r][c][i];
        int current_idx = cb_idx[r][c][i];

        for (int d_idx = 0; d_idx < 2; ++d_idx) {
            int d = (d_idx == 0) ? 1 : -1;
            
            int next_idx = (current_idx + d + 40) % 40;
            int next_r = belt_r[m][next_idx];
            int next_c = belt_c[m][next_idx];
            
            int next_dist = dist_to_exit[next_r][next_c];

            // 絶対前進ルールは維持（迷走を完全に防ぐ）
            if (next_dist >= current_dist) continue;

            long long sub_score = 0;
            long long weights[] = {1000000LL, 10000LL, 100LL};
            
            for (int offset = 1; offset <= 3; ++offset) {
                int check_id = target + offset;
                if (check_id >= 400) break;
                if (pos_r[check_id] == -1) continue;
                
                int cr = pos_r[check_id];
                int cc = pos_c[check_id];
                
                bool on_belt = false;
                int c_idx = -1;
                for (int l = 0; l < cb_cnt[cr][cc]; ++l) {
                    if (cb_m[cr][cc][l] == m) {
                        on_belt = true;
                        c_idx = cb_idx[cr][cc][l];
                        break;
                    }
                }
                
                if (on_belt) {
                    int n_idx = (c_idx + d + 40) % 40;
                    cr = belt_r[m][n_idx];
                    cc = belt_c[m][n_idx];
                }
                
                sub_score += weights[offset - 1] * dist_to_exit[cr][cc];
            }
            
            // ★乱択によるノイズを付与（最大20000のブレを作る）
            // これにより、評価値が近い「複数の正解ルート」からランダムに手が選ばれる
            sub_score += xor128() % 20000;
            
            if (sub_score < best_sub_score) {
                best_sub_score = sub_score;
                best_m = m;
                best_d = d;
            }
        }
    }
    
    if (best_m == -1) {
        return {cb_m[r][c][0], 1};
    }
    return {best_m, best_d};
}

void shift_belt(int m, int d) {
    int temp[40]; 
    for (int i = 0; i < 40; ++i) {
        temp[i] = grid[belt_r[m][i]][belt_c[m][i]];
    }
    for (int i = 0; i < 40; ++i) {
        int new_idx = (i + d + 40) % 40;
        int box_id = temp[i];
        grid[belt_r[m][new_idx]][belt_c[m][new_idx]] = box_id;
        if (box_id != -1) {
            pos_r[box_id] = belt_r[m][new_idx];
            pos_c[box_id] = belt_c[m][new_idx];
        }
    }
    
    ops_m[ops_count] = m;
    ops_d[ops_count] = d;
    ops_count++;

    if (grid[0][10] == current_target) {
        grid[0][10] = -1;
        pos_r[current_target] = -1;
        pos_c[current_target] = -1;
        current_target++;
    }
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    auto start_time = steady_clock::now();

    int dummy_N;
    if (!(cin >> dummy_N)) return 0;

    for(int i = 0; i < 400; ++i) {
        init_pos_r[i] = -1;
        init_pos_c[i] = -1;
    }

    for (int i = 0; i < 20; ++i) {
        for (int j = 0; j < 20; ++j) {
            int v;
            cin >> v;
            init_grid[i][j] = v;
            if (v != -1) {
                init_pos_r[v] = i;
                init_pos_c[v] = j;
            }
        }
    }

    init_belts();
    init_bfs();

    // --- メインシミュレーションループ ---
    int simulation_count = 0;
    while (true) {
        auto current_time = steady_clock::now();
        // 1.90秒で探索を打ち切る
        if (duration_cast<milliseconds>(current_time - start_time).count() > 1900) {
            break;
        }

        reset_state();
        simulation_count++;

        while (current_target < 400) {
            // 枝刈り：すでに過去のベスト手数を上回ったら、この盤面はハズレなので捨てる
            if (ops_count >= best_ops_count) {
                break;
            }

            if (pos_r[current_target] == -1) {
                current_target++;
                continue;
            }

            if (pos_r[current_target] == 0 && pos_c[current_target] == 10) {
                shift_belt(1, 1);
                shift_belt(1, -1);
                continue;
            }

            int target_id = current_target;
            while (current_target == target_id && (pos_r[target_id] != 0 || pos_c[target_id] != 10)) {
                if (ops_count >= best_ops_count) break;
                
                pair<int, int> best_move = get_best_move_strict(current_target);
                shift_belt(best_move.first, best_move.second);
            }
        }

        // 最後まで運べて、かつ手数が過去最小ならベストスコアを更新
        if (current_target >= 400 && ops_count < best_ops_count) {
            best_ops_count = ops_count;
            for (int i = 0; i < ops_count; ++i) {
                best_ops_m[i] = ops_m[i];
                best_ops_d[i] = ops_d[i];
            }
        }
    }

    // --- 出力フェーズ ---
    // 最後に、一番成績が良かった操作列で初期盤面を動かして、最終盤面を作る
    reset_state();
    for (int i = 0; i < best_ops_count; ++i) {
        shift_belt(best_ops_m[i], best_ops_d[i]);
    }

    cout << 20 << "\n";
    for (int m = 0; m < 20; ++m) {
        cout << 40;
        for (int i = 0; i < 40; ++i) {
            cout << " " << belt_r[m][i] << " " << belt_c[m][i];
        }
        cout << "\n";
    }
    
    cout << best_ops_count << "\n";
    for (int i = 0; i < best_ops_count; ++i) {
        cout << best_ops_m[i] << " " << best_ops_d[i] << "\n";
    }

    return 0;
}