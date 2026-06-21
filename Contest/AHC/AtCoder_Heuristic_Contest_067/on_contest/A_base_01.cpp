#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>

using namespace std;

struct Point {
    int i, j;
};

struct State {
    int mask, i, j;
};

struct Door {
    int d, i, j, g;
};

struct Switch {
    int p, q, s;
};

int dist_g[1024][20][20];

bool is_open(int g, int mask) {
    if (g == -1) return true;
    int k = g / 2;
    return ((mask >> k) & 1) == (g & 1);
}

// 経路復元なしの純粋なターン数計算
int calc_T_fast(int N, int K, const vector<string>& c, 
                const vector<vector<int>>& door_h, 
                const vector<vector<int>>& door_v, 
                const vector<vector<int>>& switch_map) {
    
    int max_mask = 1 << K;
    for (int m = 0; m < max_mask; ++m) {
        for (int i = 0; i < N; ++i) {
            for (int j = 0; j < N; ++j) {
                dist_g[m][i][j] = -1;
            }
        }
    }
    
    dist_g[0][0][0] = 0;
    queue<State> que;
    que.push({0, 0, 0});

    int di[] = {-1, 1, 0, 0};
    int dj[] = {0, 0, -1, 1};

    while (!que.empty()) {
        State curr = que.front();
        que.pop();

        int mask = curr.mask;
        int i = curr.i;
        int j = curr.j;
        int d = dist_g[mask][i][j];

        if (i == N - 1 && j == N - 1) {
            return d;
        }

        for (int dir = 0; dir < 4; ++dir) {
            int ni = i + di[dir];
            int nj = j + dj[dir];

            if (!(ni >= 0 && ni < N && nj >= 0 && nj < N)) continue;
            if (c[ni][nj] == '#') continue;

            int g = -1;
            if (di[dir] == 1)       g = door_h[i][j];
            else if (di[dir] == -1) g = door_h[ni][nj];
            else if (dj[dir] == 1)  g = door_v[i][j];
            else                    g = door_v[ni][nj];

            if (!is_open(g, mask)) continue;

            if (dist_g[mask][ni][nj] == -1) {
                dist_g[mask][ni][nj] = d + 1;
                que.push({mask, ni, nj});
            }
        }

        int s = switch_map[i][j];
        if (s != -1) {
            int nmask = mask ^ (1 << s);
            if (dist_g[nmask][i][j] == -1) {
                dist_g[nmask][i][j] = d + 1;
                que.push({nmask, i, j});
            }
        }
    }

    return 0;
}

// 経路復元関数
pair<int, vector<Point>> calc_T_with_path(
    int N, int K, const vector<string>& c,
    const vector<vector<int>>& door_h,
    const vector<vector<int>>& door_v,
    const vector<vector<int>>& switch_map) {

    vector<vector<vector<int>>> dist(1 << K, vector<vector<int>>(N, vector<int>(N, -1)));
    vector<vector<vector<State>>> parent(1 << K, vector<vector<State>>(N, vector<State>(N, {-1, -1, -1})));

    dist[0][0][0] = 0;
    queue<State> que;
    que.push({0, 0, 0});

    int di[] = {-1, 1, 0, 0};
    int dj[] = {0, 0, -1, 1};

    int goal_mask = -1;
    bool found_goal = false;

    while (!que.empty()) {
        State curr = que.front();
        que.pop();

        int mask = curr.mask;
        int i = curr.i;
        int j = curr.j;
        int d = dist[mask][i][j];

        if (i == N - 1 && j == N - 1) {
            goal_mask = mask;
            found_goal = true;
            break;
        }

        for (int dir = 0; dir < 4; ++dir) {
            int ni = i + di[dir];
            int nj = j + dj[dir];

            if (!(ni >= 0 && ni < N && nj >= 0 && nj < N)) continue;
            if (c[ni][nj] == '#') continue;

            int g = -1;
            if (di[dir] == 1)       g = door_h[i][j];
            else if (di[dir] == -1) g = door_h[ni][nj];
            else if (dj[dir] == 1)  g = door_v[i][j];
            else                    g = door_v[ni][nj];

            if (!is_open(g, mask)) continue;

            if (dist[mask][ni][nj] == -1) {
                dist[mask][ni][nj] = d + 1;
                parent[mask][ni][nj] = {mask, i, j};
                que.push({mask, ni, nj});
            }
        }

        int s = switch_map[i][j];
        if (s != -1) {
            int nmask = mask ^ (1 << s);
            if (dist[nmask][i][j] == -1) {
                dist[nmask][i][j] = d + 1;
                parent[nmask][i][j] = {mask, i, j};
                que.push({nmask, i, j});
            }
        }
    }

    if (!found_goal) return {0, {}};

    int total_turn = dist[goal_mask][N - 1][N - 1];

    vector<Point> path_points;
    State curr_state = {goal_mask, N - 1, N - 1};
    while (curr_state.i != -1) {
        if (path_points.empty() || 
            path_points.back().i != curr_state.i || 
            path_points.back().j != curr_state.j) {
            path_points.push_back({curr_state.i, curr_state.j});
        }
        curr_state = parent[curr_state.mask][curr_state.i][curr_state.j];
    }
    reverse(path_points.begin(), path_points.end());

    return {total_turn, path_points};
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    int N, M, K;
    if (!(cin >> N >> M >> K)) return 0;

    vector<string> c(N);
    for (int i = 0; i < N; ++i) {
        cin >> c[i];
    }

    cerr << "[DEBUG] 入力完了: N=" << N << ", M=" << M << ", K=" << K << "\n";

    vector<vector<int>> door_h(N, vector<int>(N, -1));
    vector<vector<int>> door_v(N, vector<int>(N, -1));
    vector<vector<int>> switch_map(N, vector<int>(N, -1));

    vector<Door> final_doors;
    vector<Switch> final_switches;

    for (int step = 0; step < K; ++step) {
        // すでに扉が上限枚数に達している場合は、新しい関所の探索を行わない
        if ((int)final_doors.size() >= M) {
            cerr << "[DEBUG] スキル " << step << " : 扉がすでに最大枚数に達しているためスキップします。\n";
            continue;
        }

        auto [current_best_T, current_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
        
        if (current_best_T == 0 || current_path.size() < 5) {
            cerr << "[DEBUG] 探索を終了します。\n";
            break;
        }

        int best_T_this_step = current_best_T;
        int best_door_d = -1, best_door_i = -1, best_door_j = -1;
        int best_switch_p = -1, best_switch_q = -1;
        int door_g = 2 * step + 1;

        for (size_t idx = 4; idx < current_path.size(); ++idx) {
            Point prev = current_path[idx - 1];
            Point curr = current_path[idx];

            Point s_pos = current_path[idx - 4];
            if (switch_map[s_pos.i][s_pos.j] != -1) continue;

            switch_map[s_pos.i][s_pos.j] = step;

            if (curr.i == prev.i + 1 && curr.j == prev.j && door_h[prev.i][prev.j] == -1) {
                door_h[prev.i][prev.j] = door_g;
                int score = calc_T_fast(N, K, c, door_h, door_v, switch_map);
                if (score > best_T_this_step) {
                    best_T_this_step = score;
                    best_door_d = 0; best_door_i = prev.i; best_door_j = prev.j;
                    best_switch_p = s_pos.i; best_switch_q = s_pos.j;
                }
                door_h[prev.i][prev.j] = -1;
            }
            else if (curr.i == prev.i - 1 && curr.j == prev.j && door_h[curr.i][curr.j] == -1) {
                door_h[curr.i][curr.j] = door_g;
                int score = calc_T_fast(N, K, c, door_h, door_v, switch_map);
                if (score > best_T_this_step) {
                    best_T_this_step = score;
                    best_door_d = 0; best_door_i = curr.i; best_door_j = curr.j;
                    best_switch_p = s_pos.i; best_switch_q = s_pos.j;
                }
                door_h[curr.i][curr.j] = -1;
            }
            else if (curr.i == prev.i && curr.j == prev.j + 1 && door_v[prev.i][prev.j] == -1) {
                door_v[prev.i][prev.j] = door_g;
                int score = calc_T_fast(N, K, c, door_h, door_v, switch_map);
                if (score > best_T_this_step) {
                    best_T_this_step = score;
                    best_door_d = 1; best_door_i = prev.i; best_door_j = prev.j;
                    best_switch_p = s_pos.i; best_switch_q = s_pos.j;
                }
                door_v[prev.i][prev.j] = -1;
            }
            else if (curr.i == prev.i && curr.j == prev.j - 1 && door_v[curr.i][curr.j] == -1) {
                door_v[curr.i][curr.j] = door_g;
                int score = calc_T_fast(N, K, c, door_h, door_v, switch_map);
                if (score > best_T_this_step) {
                    best_T_this_step = score;
                    best_door_d = 1; best_door_i = curr.i; best_door_j = curr.j;
                    best_switch_p = s_pos.i; best_switch_q = s_pos.j;
                }
                door_v[curr.i][curr.j] = -1;
            }

            switch_map[s_pos.i][s_pos.j] = -1;
        }

        if (best_door_d != -1) {
            switch_map[best_switch_p][best_switch_q] = step;
            final_switches.push_back({best_switch_p, best_switch_q, step});

            if (best_door_d == 0) door_h[best_door_i][best_door_j] = door_g;
            else                  door_v[best_door_i][best_door_j] = door_g;
            final_doors.push_back({best_door_d, best_door_i, best_door_j, door_g});

            // 周囲への拡張BFS
            queue<Point> grid_que;
            grid_que.push({best_door_i, best_door_j});

            while (!grid_que.empty() && (int)final_doors.size() < M) {
                Point curr_door = grid_que.front();
                grid_que.pop();

                int di_offset[] = {-1, 1, 0, 0};
                int dj_offset[] = {0, 0, -1, 1};

                for (int d_dir = 0; d_dir < 4; ++d_dir) {
                    // 追加直前での枚数チェック（厳密なガード）
                    if ((int)final_doors.size() >= M) break;

                    int ni = curr_door.i + di_offset[d_dir];
                    int nj = curr_door.j + dj_offset[d_dir];

                    // 1. 水平方向の拡張
                    if (ni >= 0 && ni < N - 1 && nj >= 0 && nj < N) {
                        if (c[ni][nj] == '.' && c[ni+1][nj] == '.' && door_h[ni][nj] == -1) {
                            door_h[ni][nj] = door_g;
                            if (calc_T_fast(N, K, c, door_h, door_v, switch_map) > 0) {
                                final_doors.push_back({0, ni, nj, door_g});
                                grid_que.push({ni, nj});
                            } else {
                                door_h[ni][nj] = -1;
                            }
                        }
                    }
                    // 2. 垂直方向の拡張
                    if ((int)final_doors.size() >= M) break; // 再度チェック
                    if (ni >= 0 && ni < N && nj >= 0 && nj < N - 1) {
                        if (c[ni][nj] == '.' && c[ni][nj+1] == '.' && door_v[ni][nj] == -1) {
                            door_v[ni][nj] = door_g;
                            if (calc_T_fast(N, K, c, door_h, door_v, switch_map) > 0) {
                                final_doors.push_back({1, ni, nj, door_g});
                                grid_que.push({ni, nj});
                            } else {
                                door_v[ni][nj] = -1;
                            }
                        }
                    }
                }
            }

            int current_actual_score = calc_T_fast(N, K, c, door_h, door_v, switch_map);
            cerr << "[DEBUG] スキル " << step << " 確定 -> 敷き詰め後のターン数: " << current_actual_score 
                 << " (総扉数: " << final_doors.size() << ")\n";
        } else {
            cerr << "[DEBUG] スキル " << step << " : 配置可能な場所がありませんでした。\n";
        }
    }

    // 残り枠を安全なダミー扉で埋める処理（ここでも追加直前のガードを徹底）
    int step_dummy = 0;
    while ((int)final_doors.size() < M) {
        bool dummy_placed = false;
        int dummy_g = 2 * (step_dummy % K) + 1;
        for (int i = 0; i < N && (int)final_doors.size() < M; ++i) {
            for (int j = 0; j < N && (int)final_doors.size() < M; ++j) {
                if ((int)final_doors.size() >= M) break;
                if (i < N - 1 && c[i][j] == '.' && c[i+1][j] == '.' && door_h[i][j] == -1) {
                    door_h[i][j] = dummy_g;
                    if (calc_T_fast(N, K, c, door_h, door_v, switch_map) > 0) {
                        final_doors.push_back({0, i, j, dummy_g});
                        dummy_placed = true;
                    } else {
                        door_h[i][j] = -1;
                    }
                }
                if ((int)final_doors.size() >= M) break;
                if (j < N - 1 && c[i][j] == '.' && c[i][j+1] == '.' && door_v[i][j] == -1) {
                    door_v[i][j] = dummy_g;
                    if (calc_T_fast(N, K, c, door_h, door_v, switch_map) > 0) {
                        final_doors.push_back({1, i, j, dummy_g});
                        dummy_placed = true;
                    } else {
                        door_v[i][j] = -1;
                    }
                }
            }
        }
        if (!dummy_placed) break;
        step_dummy++;
    }

    // スイッチも不足していれば10個になるまで埋める
    int switch_id = final_switches.size();
    for (int i = 0; i < N && (int)final_switches.size() < K; ++i) {
        for (int j = 0; j < N && (int)final_switches.size() < K; ++j) {
            if (c[i][j] == '.' && switch_map[i][j] == -1) {
                switch_map[i][j] = switch_id;
                final_switches.push_back({i, j, switch_id});
                switch_id++;
            }
        }
    }

    auto [final_T, _] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
    cerr << "[DEBUG] 最終結果 (turn): " << final_T << "\n";
    cerr << "[DEBUG] 出力サイズ - 扉: " << final_doors.size() << "枚, スイッチ: " << final_switches.size() << "個\n";

    cout << final_doors.size() << "\n";
    for (const auto& d : final_doors) {
        cout << d.d << " " << d.i << " " << d.j << " " << d.g << "\n";
    }

    cout << final_switches.size() << "\n";
    for (const auto& s : final_switches) {
        cout << s.p << " " << s.q << " " << s.s << "\n";
    }

    return 0;
}