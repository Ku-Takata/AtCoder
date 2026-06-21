#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <chrono>
#include <random>
#include <cmath>

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

// calc_detour_path 用のダイクストラ状態構造体
struct DetourState {
    int cost, d, mask, i, j;
    bool operator>(const DetourState& o) const {
        return cost > o.cost;
    }
};

// calc_T_fast 用のグローバル静的配列と世代管理
int dist_g[1024][20][20];
int visited_iter[1024][20][20];
int current_iter_id = 0;

// calc_T_with_path 用のグローバル静的配列と世代管理
int dist_path[1024][20][20];
State parent_path[1024][20][20];
int visited_path_iter[1024][20][20];
int current_path_iter_id = 0;

// 動的確保を避けるための使い回し用キューバッファ
State q_buf[410000];

bool is_open(int g, int mask) {
    if (g == -1) return true;
    int k = g / 2;
    return ((mask >> k) & 1) == (g & 1);
}

// 経路復元なしの純粋なターン数計算
int calc_T_fast(int N, int K, const vector<string>& c, 
                const vector<vector<int>>& door_h, 
                const vector<vector<int>>& door_v, 
                const vector<vector<int>>& switch_map,
                int threshold) {
    
    (void)K;
    current_iter_id++;
    if (current_iter_id == 0) {
        current_iter_id = 1;
        for (int m = 0; m < 1024; ++m) {
            for (int i = 0; i < 20; ++i) {
                for (int j = 0; j < 20; ++j) {
                    visited_iter[m][i][j] = 0;
                }
            }
        }
    }
    
    dist_g[0][0][0] = 0;
    visited_iter[0][0][0] = current_iter_id;
    
    int q_head = 0, q_tail = 0;
    q_buf[q_tail++] = {0, 0, 0};

    int di[] = {-1, 1, 0, 0};
    int dj[] = {0, 0, -1, 1};

    while (q_head < q_tail) {
        State curr = q_buf[q_head++];

        int mask = curr.mask;
        int i = curr.i;
        int j = curr.j;
        int d = dist_g[mask][i][j];

        if (i == N - 1 && j == N - 1) {
            if (d < threshold) return 0;
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

            if (visited_iter[mask][ni][nj] != current_iter_id) {
                visited_iter[mask][ni][nj] = current_iter_id;
                dist_g[mask][ni][nj] = d + 1;
                q_buf[q_tail++] = {mask, ni, nj};
            }
        }

        int s = switch_map[i][j];
        if (s != -1) {
            int nmask = mask ^ (1 << s);
            if (visited_iter[nmask][i][j] != current_iter_id) {
                visited_iter[nmask][i][j] = current_iter_id;
                dist_g[nmask][i][j] = d + 1;
                q_buf[q_tail++] = {nmask, i, j};
            }
        }
    }
    return 0;
}

// 案3実装用：中央ペナルティ付きの経路探索（ダイクストラ法）
pair<int, vector<Point>> calc_detour_path(
    int N, int K, const vector<string>& c,
    const vector<vector<int>>& door_h,
    const vector<vector<int>>& door_v,
    const vector<vector<int>>& switch_map) {

    current_path_iter_id++;
    if (current_path_iter_id == 0) {
        current_path_iter_id = 1;
        for (int m = 0; m < 1024; ++m) {
            for (int i = 0; i < 20; ++i) {
                for (int j = 0; j < 20; ++j) {
                    visited_path_iter[m][i][j] = 0;
                }
            }
        }
    }

    priority_queue<DetourState, vector<DetourState>, greater<DetourState>> pq;
    pq.push({0, 0, 0, 0, 0});
    
    visited_path_iter[0][0][0] = current_path_iter_id;
    dist_path[0][0][0] = 0;
    parent_path[0][0][0] = {-1, -1, -1};

    int goal_mask = -1;
    bool found_goal = false;
    
    int di[] = {-1, 1, 0, 0};
    int dj[] = {0, 0, -1, 1};

    while (!pq.empty()) {
        DetourState curr = pq.top();
        pq.pop();

        if (curr.i == N - 1 && curr.j == N - 1) {
            goal_mask = curr.mask;
            found_goal = true;
            break;
        }

        if (dist_path[curr.mask][curr.i][curr.j] < curr.cost) continue;

        for (int dir = 0; dir < 4; ++dir) {
            int ni = curr.i + di[dir];
            int nj = curr.j + dj[dir];

            if (!(ni >= 0 && ni < N && nj >= 0 && nj < N)) continue;
            if (c[ni][nj] == '#') continue;

            int g = -1;
            if (di[dir] == 1)       g = door_h[curr.i][curr.j];
            else if (di[dir] == -1) g = door_h[ni][nj];
            else if (dj[dir] == 1)  g = door_v[curr.i][curr.j];
            else                    g = door_v[ni][nj];

            if (!is_open(g, curr.mask)) continue;

            int cx = N / 2, cy = N / 2;
            int penalty = max(0, 10 - (abs(ni - cx) + abs(nj - cy))) * 5;
            int next_cost = curr.cost + 1 + penalty;

            if (visited_path_iter[curr.mask][ni][nj] != current_path_iter_id || next_cost < dist_path[curr.mask][ni][nj]) {
                visited_path_iter[curr.mask][ni][nj] = current_path_iter_id;
                dist_path[curr.mask][ni][nj] = next_cost;
                parent_path[curr.mask][ni][nj] = {curr.mask, curr.i, curr.j};
                pq.push({next_cost, curr.d + 1, curr.mask, ni, nj});
            }
        }

        int s = switch_map[curr.i][curr.j];
        if (s != -1) {
            int nmask = curr.mask ^ (1 << s);
            int next_cost = curr.cost + 1;
            if (visited_path_iter[nmask][curr.i][curr.j] != current_path_iter_id || next_cost < dist_path[nmask][curr.i][curr.j]) {
                visited_path_iter[nmask][curr.i][curr.j] = current_path_iter_id;
                dist_path[nmask][curr.i][curr.j] = next_cost;
                parent_path[nmask][curr.i][curr.j] = {curr.mask, curr.i, curr.j};
                pq.push({next_cost, curr.d + 1, nmask, curr.i, curr.j});
            }
        }
    }

    if (!found_goal) return {0, {}};

    vector<Point> path_points;
    State curr_state = {goal_mask, N - 1, N - 1};
    while (curr_state.i != -1) {
        if (path_points.empty() || 
            path_points.back().i != curr_state.i || 
            path_points.back().j != curr_state.j) {
            path_points.push_back({curr_state.i, curr_state.j});
        }
        curr_state = parent_path[curr_state.mask][curr_state.i][curr_state.j];
    }
    reverse(path_points.begin(), path_points.end());

    int actual_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, 0);
    return {actual_T, path_points};
}

pair<int, vector<Point>> calc_T_with_path(
    int N, int K, const vector<string>& c,
    const vector<vector<int>>& door_h,
    const vector<vector<int>>& door_v,
    const vector<vector<int>>& switch_map) {

    current_path_iter_id++;
    if (current_path_iter_id == 0) {
        current_path_iter_id = 1;
        for (int m = 0; m < 1024; ++m) {
            for (int i = 0; i < 20; ++i) {
                for (int j = 0; j < 20; ++j) {
                    visited_path_iter[m][i][j] = 0;
                }
            }
        }
    }

    dist_path[0][0][0] = 0;
    visited_path_iter[0][0][0] = current_path_iter_id;
    parent_path[0][0][0] = {-1, -1, -1};
    
    int q_head = 0, q_tail = 0;
    q_buf[q_tail++] = {0, 0, 0};

    int di[] = {-1, 1, 0, 0};
    int dj[] = {0, 0, -1, 1};

    int goal_mask = -1;
    bool found_goal = false;

    while (q_head < q_tail) {
        State curr = q_buf[q_head++];

        int mask = curr.mask;
        int i = curr.i;
        int j = curr.j;
        int d = dist_path[mask][i][j];

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

            if (visited_path_iter[mask][ni][nj] != current_path_iter_id) {
                visited_path_iter[mask][ni][nj] = current_path_iter_id;
                dist_path[mask][ni][nj] = d + 1;
                parent_path[mask][ni][nj] = {mask, i, j};
                q_buf[q_tail++] = {mask, ni, nj};
            }
        }

        int s = switch_map[i][j];
        if (s != -1) {
            int nmask = mask ^ (1 << s);
            if (visited_path_iter[nmask][i][j] != current_path_iter_id) {
                visited_path_iter[nmask][i][j] = current_path_iter_id;
                dist_path[nmask][i][j] = d + 1;
                parent_path[nmask][i][j] = {mask, i, j};
                q_buf[q_tail++] = {nmask, i, j};
            }
        }
    }

    if (!found_goal) return {0, {}};

    int total_turn = dist_path[goal_mask][N - 1][N - 1];

    vector<Point> path_points;
    State curr_state = {goal_mask, N - 1, N - 1};
    while (curr_state.i != -1) {
        if (path_points.empty() || 
            path_points.back().i != curr_state.i || 
            path_points.back().j != curr_state.j) {
            path_points.push_back({curr_state.i, curr_state.j});
        }
        curr_state = parent_path[curr_state.mask][curr_state.i][curr_state.j];
    }
    reverse(path_points.begin(), path_points.end());

    return {total_turn, path_points};
}

int main() {
    auto start_time = chrono::steady_clock::now();

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
    vector<Door> init_doors;

    // 初期解構築フェーズ
    for (int step = 0; step < K; ++step) {
        auto [current_best_T, current_path] = calc_detour_path(N, K, c, door_h, door_v, switch_map);
        if (current_best_T == 0 || current_path.size() < 5) break;

        int best_T_this_step = calc_T_fast(N, K, c, door_h, door_v, switch_map, 0);
        int best_door_d = -1, best_door_i = -1, best_door_j = -1;
        int best_switch_p = -1, best_switch_q = -1;
        int door_g = 2 * step + 1;

        for (int idx = (int)current_path.size() - 1; idx >= 2; --idx) {
            Point prev = current_path[idx - 1];
            Point curr = current_path[idx];

            vector<Point> sw_cands;
            for(int r = 0; r < N; ++r) {
                for(int l = 0; l < N; ++l) {
                    if(c[r][l] == '.' && switch_map[r][l] == -1) {
                        sw_cands.push_back({r, l});
                    }
                }
            }
            sort(sw_cands.begin(), sw_cands.end(), [&](const Point& a, const Point& b) {
                int dist_a = (abs(a.i - prev.i) + abs(a.j - prev.j)) + (abs(a.i - N/2) + abs(a.j - N/2));
                int dist_b = (abs(b.i - prev.i) + abs(b.j - prev.j)) + (abs(b.i - N/2) + abs(b.j - N/2));
                return dist_a > dist_b;
            });

            int cand_limit = min(5, (int)sw_cands.size());

            auto test_door_and_switches = [&](int d, int i, int j) {
                for(int k = 0; k < cand_limit; ++k) {
                    Point s_pos = sw_cands[k];
                    switch_map[s_pos.i][s_pos.j] = step;
                    
                    int score = calc_T_fast(N, K, c, door_h, door_v, switch_map, best_T_this_step);
                    if (score > best_T_this_step) {
                        best_T_this_step = score;
                        best_door_d = d; best_door_i = i; best_door_j = j;
                        best_switch_p = s_pos.i; best_switch_q = s_pos.j;
                    }
                    switch_map[s_pos.i][s_pos.j] = -1;
                }
            };

            if (curr.i == prev.i + 1 && curr.j == prev.j && door_h[prev.i][prev.j] == -1) {
                door_h[prev.i][prev.j] = door_g;
                test_door_and_switches(0, prev.i, prev.j);
                door_h[prev.i][prev.j] = -1;
            }
            else if (curr.i == prev.i - 1 && curr.j == prev.j && door_h[curr.i][curr.j] == -1) {
                door_h[curr.i][curr.j] = door_g;
                test_door_and_switches(0, curr.i, curr.j);
                door_h[curr.i][curr.j] = -1;
            }
            else if (curr.i == prev.i && curr.j == prev.j + 1 && door_v[prev.i][prev.j] == -1) {
                door_v[prev.i][prev.j] = door_g;
                test_door_and_switches(1, prev.i, prev.j);
                door_v[prev.i][prev.j] = -1;
            }
            else if (curr.i == prev.i && curr.j == prev.j - 1 && door_v[curr.i][curr.j] == -1) {
                door_v[curr.i][curr.j] = door_g;
                test_door_and_switches(1, curr.i, curr.j);
                door_v[curr.i][curr.j] = -1;
            }
        }

        if (best_door_d != -1) {
            switch_map[best_switch_p][best_switch_q] = step;
            if (best_door_d == 0) door_h[best_door_i][best_door_j] = door_g;
            else                  door_v[best_door_i][best_door_j] = door_g;
            init_doors.push_back({best_door_d, best_door_i, best_door_j, door_g});
            cerr << "[DEBUG] 初期解 Step " << step << " 確定: T=" << best_T_this_step << "\n";
        }
    }

    // 焼きなまし法のセットアップ
    mt19937 engine(42);

    auto [current_T, current_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
    int best_T = current_T;

    cerr << "[DEBUG] 初期解構築完了(大回り経路+隔離スイッチ): 初期ターン数 = " << best_T 
         << " (初期扉数: " << init_doors.size() << ")\n";

    vector<vector<int>> best_door_h = door_h;
    vector<vector<int>> best_door_v = door_v;
    vector<vector<int>> best_switch_map = switch_map;

    int current_door_count = init_doors.size();
    long long iter = 0;

    const double T_start = 2.0;
    const double T_end = 0.01;
    const int TIME_LIMIT_MS = 1650;

    // 焼きなましループ
    while (true) {
        iter++;
        
        double temp = T_start;
        int elapsed_ms = 0;
        if (iter % 20 == 0) {
            auto now = chrono::steady_clock::now();
            elapsed_ms = chrono::duration_cast<chrono::milliseconds>(now - start_time).count();
            if (elapsed_ms > TIME_LIMIT_MS) break;
            
            double time_ratio = (double)elapsed_ms / TIME_LIMIT_MS;
            temp = T_start * pow(T_end / T_start, time_ratio);
        }

        if (iter % 50 == 0) {
            auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
            if (!updated_path.empty()) current_path = updated_path;
        }

        int i = 0, j = 0;
        if (current_path.empty()) {
            i = engine() % N;
            j = engine() % N;
        } else {
            int path_idx = engine() % current_path.size();
            i = current_path[path_idx].i;
            j = current_path[path_idx].j;
            if (engine() % 3 == 0) {
                int di_offset[] = {-1, 1, 0, 0};
                int dj_offset[] = {0, 0, -1, 1};
                int dir = engine() % 4;
                int ni = i + di_offset[dir];
                int nj = j + dj_offset[dir];
                if (ni >= 0 && ni < N && nj >= 0 && nj < N) {
                    i = ni;
                    j = nj;
                }
            }
        }

        int mode = engine() % 6;
        bool accept = false;
        int delta = 0;
        int threshold = max(1, current_T - (int)(temp * 3.0) - 2);

        if (mode == 0) {
            int d = engine() % 2;
            if (d == 0) {
                if (i >= N - 1) continue;
                if (c[i][j] == '#' || c[i+1][j] == '#') continue;

                int old_g = door_h[i][j];
                int old_count = current_door_count;

                if (old_g != -1) {
                    if (engine() % 2 == 0) { door_h[i][j] = -1; current_door_count--; }
                    else { door_h[i][j] = engine() % (2 * K); }
                } else {
                    if (current_door_count >= M) continue;
                    door_h[i][j] = engine() % (2 * K); current_door_count++;
                }

                int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
                delta = next_T - current_T;
                if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;

                if (accept) {
                    current_T = next_T;
                    if (current_T > best_T) {
                        best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                        auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                        if (!updated_path.empty()) current_path = updated_path;
                    }
                } else { door_h[i][j] = old_g; current_door_count = old_count; }
            } else {
                if (j >= N - 1) continue;
                if (c[i][j] == '#' || c[i][j+1] == '#') continue;

                int old_g = door_v[i][j];
                int old_count = current_door_count;

                if (old_g != -1) {
                    if (engine() % 2 == 0) { door_v[i][j] = -1; current_door_count--; }
                    else { door_v[i][j] = engine() % (2 * K); }
                } else {
                    if (current_door_count >= M) continue;
                    door_v[i][j] = engine() % (2 * K); current_door_count++;
                }

                int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
                delta = next_T - current_T;
                if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;

                if (accept) {
                    current_T = next_T;
                    if (current_T > best_T) {
                        best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                        auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                        if (!updated_path.empty()) current_path = updated_path;
                    }
                } else { door_v[i][j] = old_g; current_door_count = old_count; }
            }
        } 
        else if (mode == 1) {
            if (c[i][j] == '#') continue;
            int old_s = switch_map[i][j];

            if (old_s != -1) {
                if (engine() % 2 == 0) { switch_map[i][j] = -1; }
                else { switch_map[i][j] = engine() % K; }
            } else { switch_map[i][j] = engine() % K; }

            int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
            delta = next_T - current_T;
            if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;

            if (accept) {
                current_T = next_T;
                if (current_T > best_T) {
                    best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                    auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                    if (!updated_path.empty()) current_path = updated_path;
                }
            } else { switch_map[i][j] = old_s; }
        } 
        else if (mode == 2) {
            struct LocatedDoor { int d, i, j, g; };
            vector<LocatedDoor> existing_doors;
            for (int r = 0; r < N; ++r) {
                for (int l = 0; l < N; ++l) {
                    if (door_h[r][l] != -1) existing_doors.push_back({0, r, l, door_h[r][l]});
                    if (door_v[r][l] != -1) existing_doors.push_back({1, r, l, door_v[r][l]});
                }
            }
            if (existing_doors.empty()) continue;

            int idx = engine() % existing_doors.size();
            LocatedDoor target = existing_doors[idx];

            if (target.d == 0) door_h[target.i][target.j] = -1;
            else               door_v[target.i][target.j] = -1;

            int nd = engine() % 2;
            bool valid_move = false;
            if (nd == 0) {
                if (i < N - 1 && c[i][j] != '#' && c[i+1][j] != '#' && door_h[i][j] == -1) {
                    door_h[i][j] = target.g; valid_move = true;
                }
            } else {
                if (j < N - 1 && c[i][j] != '#' && c[i][j+1] != '#' && door_v[i][j] == -1) {
                    door_v[i][j] = target.g; valid_move = true;
                }
            }

            if (!valid_move) {
                if (target.d == 0) door_h[target.i][target.j] = target.g;
                else               door_v[target.i][target.j] = target.g;
                continue;
            }

            int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
            delta = next_T - current_T;
            if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;

            if (accept) {
                current_T = next_T;
                if (current_T > best_T) {
                    best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                    auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                    if (!updated_path.empty()) current_path = updated_path;
                }
            } else {
                if (nd == 0) door_h[i][j] = -1; else door_v[i][j] = -1;
                if (target.d == 0) door_h[target.i][target.j] = target.g; else door_v[target.i][target.j] = target.g;
            }
        } 
        else if (mode == 3) {
            struct LocatedSwitch { int i, j, s; };
            vector<LocatedSwitch> existing_switches;
            for (int r = 0; r < N; ++r) {
                for (int l = 0; l < N; ++l) {
                    if (switch_map[r][l] != -1) existing_switches.push_back({r, l, switch_map[r][l]});
                }
            }
            if (existing_switches.empty()) continue;

            int idx = engine() % existing_switches.size();
            LocatedSwitch target = existing_switches[idx];

            if (c[i][j] == '#' || switch_map[i][j] != -1) continue;

            switch_map[target.i][target.j] = -1;
            switch_map[i][j] = target.s;

            int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
            delta = next_T - current_T;
            if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;

            if (accept) {
                current_T = next_T;
                if (current_T > best_T) {
                    best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                    auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                    if (!updated_path.empty()) current_path = updated_path;
                }
            } else {
                switch_map[i][j] = -1;
                switch_map[target.i][target.j] = target.s;
            }
        }
        else if (mode == 4) {
            int k = engine() % K;
            for (int r = 0; r < N; ++r) {
                for (int l = 0; l < N; ++l) {
                    if (door_h[r][l] != -1 && door_h[r][l] / 2 == k) door_h[r][l] ^= 1;
                    if (door_v[r][l] != -1 && door_v[r][l] / 2 == k) door_v[r][l] ^= 1;
                }
            }
            
            int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
            delta = next_T - current_T;
            if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;
            
            if (accept) {
                current_T = next_T;
                if (current_T > best_T) {
                    best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                    auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                    if (!updated_path.empty()) current_path = updated_path;
                }
            } else {
                for (int r = 0; r < N; ++r) {
                    for (int l = 0; l < N; ++l) {
                        if (door_h[r][l] != -1 && door_h[r][l] / 2 == k) door_h[r][l] ^= 1;
                        if (door_v[r][l] != -1 && door_v[r][l] / 2 == k) door_v[r][l] ^= 1;
                    }
                }
            }
        }
        else if (mode == 5) {
            int target_k = engine() % K;
            int di_offset = (engine() % 7) - 3;
            int dj_offset = (engine() % 7) - 3;
            if (di_offset == 0 && dj_offset == 0) continue;
            
            auto backup_door_h = door_h;
            auto backup_door_v = door_v;
            auto backup_switch_map = switch_map;
            
            bool can_move = true;
            vector<pair<int, int>> sw_pos, dh_pos, dv_pos;
            
            for (int r = 0; r < N; ++r) {
                for (int l = 0; l < N; ++l) {
                    if (switch_map[r][l] == target_k) { sw_pos.push_back({r, l}); switch_map[r][l] = -1; }
                    if (door_h[r][l] != -1 && door_h[r][l] / 2 == target_k) { dh_pos.push_back({r, l}); door_h[r][l] = -1; }
                    if (door_v[r][l] != -1 && door_v[r][l] / 2 == target_k) { dv_pos.push_back({r, l}); door_v[r][l] = -1; }
                }
            }
            
            if (sw_pos.empty() && dh_pos.empty() && dv_pos.empty()) {
                door_h = backup_door_h; door_v = backup_door_v; switch_map = backup_switch_map;
                continue;
            }

            for (auto [r, l] : sw_pos) {
                int nr = r + di_offset, nl = l + dj_offset;
                if (nr < 0 || nr >= N || nl < 0 || nl >= N || c[nr][nl] == '#' || switch_map[nr][nl] != -1) { can_move = false; break; }
                switch_map[nr][nl] = target_k;
            }
            for (auto [r, l] : dh_pos) {
                if (!can_move) break;
                int nr = r + di_offset, nl = l + dj_offset;
                if (nr < 0 || nr >= N - 1 || nl < 0 || nl >= N || c[nr][nl] == '#' || c[nr+1][nl] == '#' || door_h[nr][nl] != -1) { can_move = false; break; }
                door_h[nr][nl] = backup_door_h[r][l];
            }
            for (auto [r, l] : dv_pos) {
                if (!can_move) break;
                int nr = r + di_offset, nl = l + dj_offset;
                if (nr < 0 || nr >= N || nl < 0 || nl >= N - 1 || c[nr][nl] == '#' || c[nr][nl+1] == '#' || door_v[nr][nl] != -1) { can_move = false; break; }
                door_v[nr][nl] = backup_door_v[r][l];
            }

            if (!can_move) {
                door_h = backup_door_h; door_v = backup_door_v; switch_map = backup_switch_map;
                continue;
            }
            
            int next_T = calc_T_fast(N, K, c, door_h, door_v, switch_map, threshold);
            delta = next_T - current_T;
            if (next_T > 0 && (delta >= 0 || (double)engine() / engine.max() < exp((double)delta / temp))) accept = true;
            
            if (accept) {
                current_T = next_T;
                if (current_T > best_T) {
                    best_T = current_T; best_door_h = door_h; best_door_v = door_v; best_switch_map = switch_map;
                    auto [_, updated_path] = calc_T_with_path(N, K, c, door_h, door_v, switch_map);
                    if (!updated_path.empty()) current_path = updated_path;
                }
            } else {
                door_h = backup_door_h; door_v = backup_door_v; switch_map = backup_switch_map;
            }
        }
    }

    door_h = best_door_h;
    door_v = best_door_v;
    switch_map = best_switch_map;

    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) {
            if (door_h[i][j] != -1) final_doors.push_back({0, i, j, door_h[i][j]});
            if (door_v[i][j] != -1) final_doors.push_back({1, i, j, door_v[i][j]});
            if (switch_map[i][j] != -1) final_switches.push_back({i, j, switch_map[i][j]});
        }
    }

    // 修正箇所：ダミー扉配置処理における厳密な時間切れチェックの導入
    int step_dummy = 0;
    bool time_up = false;
    while ((int)final_doors.size() < M && !time_up) {
        bool dummy_placed = false;
        int dummy_g = 2 * (step_dummy % K) + 1;
        for (int i = 0; i < N && (int)final_doors.size() < M && !time_up; ++i) {
            for (int j = 0; j < N && (int)final_doors.size() < M && !time_up; ++j) {
                
                // 最深部で1マスごとに時間をチェックし、超過した瞬間に即座にループを抜ける
                if (chrono::duration_cast<chrono::milliseconds>(chrono::steady_clock::now() - start_time).count() > TIME_LIMIT_MS + 200) {
                    time_up = true;
                    break;
                }

                if (i < N - 1 && c[i][j] == '.' && c[i+1][j] == '.' && door_h[i][j] == -1) {
                    door_h[i][j] = dummy_g;
                    if (calc_T_fast(N, K, c, door_h, door_v, switch_map, 1) > 0) {
                        final_doors.push_back({0, i, j, dummy_g});
                        dummy_placed = true;
                    } else {
                        door_h[i][j] = -1;
                    }
                }
                if ((int)final_doors.size() >= M) break;

                if (j < N - 1 && c[i][j] == '.' && c[i][j+1] == '.' && door_v[i][j] == -1) {
                    door_v[i][j] = dummy_g;
                    if (calc_T_fast(N, K, c, door_h, door_v, switch_map, 1) > 0) {
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