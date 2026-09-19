#include <iostream>
#include <vector>
#include <queue>
#include <tuple>
#include <chrono>
#include <algorithm>
#include <map>
#include <random>

using namespace std;

const int INF = 1e9;
const int WIDTHS[5] = {1, 3, 5, 7, 9};

struct Brick {
    int x, y, l;
    bool operator==(const Brick& o) const {
        return x == o.x && y == o.y && l == o.l;
    }
};

mt19937 rng(42);

const int MAX_W = 60;
const int MAX_H = 40;
int dist_buf[MAX_W][MAX_H + 1];
int token_buf[MAX_W][MAX_H + 1];
int cur_token = 0;

struct ParentInfo {
    int px, py, bx, by, bl;
} parent_buf[MAX_W][MAX_H + 1];

bool is_valid(const vector<Brick>& bricks, const vector<pair<int, int>>& ab, int W, int H, int K) {
    if (bricks.empty()) return false;
    vector<vector<int>> grid(W, vector<int>(H, -1));
    for (size_t i = 0; i < bricks.size(); ++i) {
        const auto& b = bricks[i];
        if (b.l != 1 && b.l != 3 && b.l != 5 && b.l != 7 && b.l != 9) return false;
        if (b.x < 0 || b.x + b.l > W || b.y < 0 || b.y >= H) return false;
        for (int dx = 0; dx < b.l; ++dx) {
            if (grid[b.x + dx][b.y] != -1) return false;
            grid[b.x + dx][b.y] = i;
        }
    }
    for (const auto& b : bricks) {
        if (b.y > 0) {
            int mid_x = b.x + (b.l - 1) / 2;
            if (grid[mid_x][b.y - 1] == -1) return false;
        }
    }
    for (int i = 0; i < K; ++i) {
        if (grid[ab[i].first][ab[i].second] == -1) return false;
    }
    return true;
}

void prune_solution(vector<Brick>& bricks, const vector<pair<int, int>>& ab, 
                    const map<int, int>& cost, int W, int H, int K) {
    bool changed = true;
    while (changed) {
        changed = false;
        int n = bricks.size();
        vector<vector<int>> cell_brick(W, vector<int>(H, -1));
        for (int i = 0; i < n; ++i) {
            for (int dx = 0; dx < bricks[i].l; ++dx) {
                cell_brick[bricks[i].x + dx][bricks[i].y] = i;
            }
        }

        vector<vector<int>> needed_support_x(n);
        for (int i = 0; i < n; ++i) {
            if (bricks[i].y > 0) {
                int mid_x = bricks[i].x + (bricks[i].l - 1) / 2;
                int p = cell_brick[mid_x][bricks[i].y - 1];
                if (p != -1) needed_support_x[p].push_back(mid_x);
            }
        }

        vector<vector<int>> needed_hole_x(n);
        for (int h = 0; h < K; ++h) {
            int hx = ab[h].first, hy = ab[h].second;
            int b = cell_brick[hx][hy];
            if (b != -1) needed_hole_x[b].push_back(hx);
        }

        for (int i = 0; i < n; ++i) {
            if (bricks[i].l <= 1) continue;
            int cur_l = bricks[i].l;
            int mid_x = bricks[i].x + (cur_l - 1) / 2;

            for (int L_new : WIDTHS) {
                if (L_new >= cur_l) continue;
                int new_bx = mid_x - (L_new - 1) / 2;
                int new_rx = new_bx + L_new - 1;

                bool supp_ok = true;
                for (int sx : needed_support_x[i]) {
                    if (sx < new_bx || sx > new_rx) { supp_ok = false; break; }
                }
                if (!supp_ok) continue;

                bool hole_ok = true;
                for (int hx : needed_hole_x[i]) {
                    if (hx < new_bx || hx > new_rx) { hole_ok = false; break; }
                }
                if (!hole_ok) continue;

                bricks[i].x = new_bx;
                bricks[i].l = L_new;
                changed = true;
                break;
            }
        }

        vector<bool> alive(n, true);
        for (int i = 0; i < n; ++i) {
            if (needed_support_x[i].empty() && needed_hole_x[i].empty()) {
                alive[i] = false;
                changed = true;
            }
        }
        if (changed) {
            vector<Brick> next_bricks;
            for (int i = 0; i < n; ++i) {
                if (alive[i]) next_bricks.push_back(bricks[i]);
            }
            bricks = next_bricks;
        }
    }
}

void merge_bricks(vector<Brick>& bricks, const vector<pair<int, int>>& ab,
                  const map<int, int>& cost, int W, int H, int K) {
    bool changed = true;
    while (changed) {
        changed = false;
        int n = bricks.size();
        vector<vector<int>> by_y(H);
        for (int i = 0; i < n; ++i) by_y[bricks[i].y].push_back(i);

        vector<vector<int>> cell_brick(W, vector<int>(H, -1));
        for (int i = 0; i < n; ++i) {
            for (int dx = 0; dx < bricks[i].l; ++dx) {
                cell_brick[bricks[i].x + dx][bricks[i].y] = i;
            }
        }

        for (int y = 0; y < H; ++y) {
            if (by_y[y].size() < 2) continue;
            auto& row = by_y[y];
            sort(row.begin(), row.end(), [&](int a, int b) { return bricks[a].x < bricks[b].x; });

            for (size_t idx = 0; idx + 1 < row.size(); ++idx) {
                int i1 = row[idx];
                int i2 = row[idx + 1];
                if (i1 == -1 || i2 == -1) continue;

                int l1 = bricks[i1].l, x1 = bricks[i1].x;
                int l2 = bricks[i2].l, x2 = bricks[i2].x;
                int r2 = x2 + l2 - 1;

                int min_x = x1;
                int max_x = r2;
                int span = max_x - min_x + 1;

                for (int L_new : WIDTHS) {
                    if (L_new < span) continue;
                    if (cost.at(L_new) >= cost.at(l1) + cost.at(l2)) continue;

                    for (int mid = (L_new - 1) / 2; mid < W - (L_new - 1) / 2; ++mid) {
                        int bx = mid - (L_new - 1) / 2;
                        int rx = bx + L_new - 1;
                        if (bx > min_x || rx < max_x) continue;

                        if (y > 0 && cell_brick[mid][y - 1] == -1) continue;

                        bool conflict = false;
                        for (int x = bx; x <= rx; ++x) {
                            if (cell_brick[x][y] != -1 && cell_brick[x][y] != i1 && cell_brick[x][y] != i2) {
                                conflict = true;
                                break;
                            }
                        }
                        if (conflict) continue;

                        bricks[i1].x = bx;
                        bricks[i1].l = L_new;
                        bricks[i2].l = 0;
                        row[idx + 1] = -1;
                        changed = true;
                        goto next_merge;
                    }
                }
            }
            next_merge:;
        }

        if (changed) {
            vector<Brick> next_bricks;
            for (const auto& b : bricks) {
                if (b.l > 0) next_bricks.push_back(b);
            }
            bricks = next_bricks;
        }
    }
}

void branch_reconnect(vector<Brick>& bricks, const vector<pair<int, int>>& ab,
                      const map<int, int>& cost, int W, int H, int K) {
    auto calc_cost = [&](const vector<Brick>& b_list) {
        int tot = 0;
        for (const auto& b : b_list) tot += cost.at(b.l);
        return tot;
    };

    bool improved = true;
    while (improved) {
        improved = false;
        int n = bricks.size();
        vector<vector<int>> cell_brick(W, vector<int>(H, -1));
        for (int i = 0; i < n; ++i) {
            for (int dx = 0; dx < bricks[i].l; ++dx) {
                cell_brick[bricks[i].x + dx][bricks[i].y] = i;
            }
        }

        vector<int> parent_of(n, -1);
        for (int i = 0; i < n; ++i) {
            if (bricks[i].y > 0) {
                int mid_x = bricks[i].x + (bricks[i].l - 1) / 2;
                int p = cell_brick[mid_x][bricks[i].y - 1];
                parent_of[i] = p;
            }
        }

        for (int i = 0; i < n; ++i) {
            if (bricks[i].y == 0) continue;
            int cur_parent = parent_of[i];
            if (cur_parent == -1) continue;

            int mid_x = bricks[i].x + (bricks[i].l - 1) / 2;
            int y = bricks[i].y;

            for (int other = 0; other < n; ++other) {
                if (other == cur_parent || other == i) continue;
                if (bricks[other].y != y - 1) continue;

                bool can_support = (bricks[other].x <= mid_x && mid_x < bricks[other].x + bricks[other].l);
                int new_other_L = -1;
                int new_other_bx = -1;
                if (!can_support) {
                    int other_mid = bricks[other].x + (bricks[other].l - 1) / 2;
                    int dist = abs(mid_x - other_mid);
                    for (int L : WIDTHS) {
                        if ((L - 1) / 2 >= dist) {
                            int bx = other_mid - (L - 1) / 2;
                            int rx = bx + L - 1;
                            if (bx <= min(bricks[other].x, mid_x) && rx >= max(bricks[other].x + bricks[other].l - 1, mid_x)) {
                                if (bx >= 0 && rx < W) {
                                    bool conf = false;
                                    for (int x = bx; x <= rx; ++x) {
                                        if (cell_brick[x][y - 1] != -1 && cell_brick[x][y - 1] != other) {
                                            conf = true; break;
                                        }
                                    }
                                    if (!conf) {
                                        new_other_L = L;
                                        new_other_bx = bx;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if (can_support || new_other_L != -1) {
                    vector<Brick> test_bricks = bricks;
                    if (new_other_L != -1) {
                        test_bricks[other].x = new_other_bx;
                        test_bricks[other].l = new_other_L;
                    }

                    prune_solution(test_bricks, ab, cost, W, H, K);

                    if (is_valid(test_bricks, ab, W, H, K)) {
                        if (calc_cost(test_bricks) < calc_cost(bricks)) {
                            bricks = test_bricks;
                            improved = true;
                            goto next_round;
                        }
                    }
                }
            }
        }
        next_round:;
    }
}

// Top-Down ダイクストラ
vector<Brick> run_top_down_dijkstra(int W, int H, int K, const map<int, int>& cost, 
                                   vector<pair<int, int>> holes, int hole_bonus, double noise_scale) {
    sort(holes.begin(), holes.end(), [&](const pair<int, int>& a, const pair<int, int>& b) {
        if (a.second != b.second) return a.second > b.second;
        return a.first < b.first;
    });

    vector<vector<bool>> grid(W, vector<bool>(H, false));
    vector<vector<bool>> is_hole_grid(W, vector<bool>(H, false));
    for (const auto& h : holes) is_hole_grid[h.first][h.second] = true;

    vector<Brick> ans;

    for (const auto& target_hole : holes) {
        int thx = target_hole.first, thy = target_hole.second;
        if (grid[thx][thy]) continue;

        cur_token++;
        priority_queue<tuple<int, int, int>, vector<tuple<int, int, int>>, greater<tuple<int, int, int>>> pq;

        for (int x = 0; x < W; ++x) {
            if (!grid[x][0]) {
                dist_buf[x][0] = 0; token_buf[x][0] = cur_token;
                pq.push({0, x, -1});
            }
        }
        for (int x = 0; x < W; ++x) {
            for (int y = 0; y < H; ++y) {
                if (grid[x][y] && y + 1 < H && !grid[x][y + 1]) {
                    dist_buf[x][y + 1] = 0; token_buf[x][y + 1] = cur_token;
                    pq.push({0, x, y});
                }
            }
        }

        bool reached = false;
        int reach_x = -1, reach_y = -1;

        while (!pq.empty()) {
            auto [d, x, y] = pq.top();
            pq.pop();
            if (token_buf[x][y + 1] == cur_token && d > dist_buf[x][y + 1]) continue;

            if (y == thy && x == thx && d > 0) {
                reach_x = x; reach_y = y;
                reached = true;
                break;
            }

            if (y >= thy) continue;
            int ny = y + 1;

            for (int L : WIDTHS) {
                int rad = (L - 1) / 2;
                int bx = x - rad;
                if (bx < 0 || bx + L > W) continue;

                bool overlap = false;
                for (int dx = 0; dx < L; ++dx) {
                    if (grid[bx + dx][ny]) { overlap = true; break; }
                }
                if (overlap) continue;

                int bonus_count = 0;
                for (int dx = 0; dx < L; ++dx) {
                    if (is_hole_grid[bx + dx][ny] && !grid[bx + dx][ny]) bonus_count++;
                }
                int base_c = cost.at(L) - bonus_count * hole_bonus;
                int noise = (noise_scale > 0 ? (int)(rng() % (int)(noise_scale * 10 + 1)) - (int)(noise_scale * 5) : 0);
                int edge_cost = max(1, base_c * 10 + noise);
                int next_dist = d + edge_cost;

                for (int dx = 0; dx < L; ++dx) {
                    int nx = bx + dx;
                    if (token_buf[nx][ny + 1] != cur_token || next_dist < dist_buf[nx][ny + 1]) {
                        dist_buf[nx][ny + 1] = next_dist;
                        token_buf[nx][ny + 1] = cur_token;
                        parent_buf[nx][ny + 1] = {x, y, bx, ny, L};
                        pq.push({next_dist, nx, ny});
                    }
                }
            }
        }

        if (!reached) continue;

        vector<Brick> path_bricks;
        int cx = reach_x, cy = reach_y;
        while (parent_buf[cx][cy + 1].px != -1 || parent_buf[cx][cy + 1].by != -1) {
            auto p = parent_buf[cx][cy + 1];
            path_bricks.push_back({p.bx, p.by, p.bl});
            if (p.py == -1 || grid[p.px][p.py]) break;
            cx = p.px; cy = p.py;
        }
        reverse(path_bricks.begin(), path_bricks.end());

        for (const auto& b : path_bricks) {
            ans.push_back(b);
            for (int dx = 0; dx < b.l; ++dx) {
                grid[b.x + dx][b.y] = true;
            }
        }
    }

    prune_solution(ans, holes, cost, W, H, K);
    merge_bricks(ans, holes, cost, W, H, K);
    branch_reconnect(ans, holes, cost, W, H, K);
    prune_solution(ans, holes, cost, W, H, K);
    return ans;
}

// 局所探索（再配線）
void local_search_fast(vector<Brick>& cur_bricks, const vector<pair<int, int>>& ab,
                       const map<int, int>& cost, const vector<vector<bool>>& is_hole,
                       int W, int H, int K, chrono::steady_clock::time_point start_time, double time_limit_ms) {
    
    auto calc_total_cost = [&](const vector<Brick>& b_list) {
        int tot = 0;
        for (const auto& b : b_list) tot += cost.at(b.l);
        return tot;
    };

    int best_cost = calc_total_cost(cur_bricks);
    vector<Brick> best_bricks = cur_bricks;

    int iter = 0;
    while (true) {
        if ((iter & 7) == 0) {
            auto now = chrono::steady_clock::now();
            double el = chrono::duration<double, milli>(now - start_time).count();
            if (el > time_limit_ms) break;
        }
        iter++;

        int num_pick = 1 + rng() % 2;
        vector<int> picked_holes;
        for (int i = 0; i < num_pick; ++i) picked_holes.push_back(rng() % K);

        vector<vector<int>> cell_brick(W, vector<int>(H, -1));
        for (size_t i = 0; i < cur_bricks.size(); ++i) {
            for (int dx = 0; dx < cur_bricks[i].l; ++dx) {
                cell_brick[cur_bricks[i].x + dx][cur_bricks[i].y] = i;
            }
        }

        vector<bool> remove_brick(cur_bricks.size(), false);
        for (int h_idx : picked_holes) {
            int b = cell_brick[ab[h_idx].first][ab[h_idx].second];
            if (b != -1) remove_brick[b] = true;
        }

        bool expanded = true;
        while (expanded) {
            expanded = false;
            for (size_t i = 0; i < cur_bricks.size(); ++i) {
                if (remove_brick[i]) continue;
                if (cur_bricks[i].y > 0) {
                    int mid_x = cur_bricks[i].x + (cur_bricks[i].l - 1) / 2;
                    int p = cell_brick[mid_x][cur_bricks[i].y - 1];
                    if (p != -1 && remove_brick[p]) {
                        remove_brick[i] = true;
                        expanded = true;
                    }
                }
            }
        }

        vector<Brick> rem_bricks;
        for (size_t i = 0; i < cur_bricks.size(); ++i) {
            if (!remove_brick[i]) rem_bricks.push_back(cur_bricks[i]);
        }

        vector<vector<bool>> grid(W, vector<bool>(H, false));
        for (const auto& b : rem_bricks) {
            for (int dx = 0; dx < b.l; ++dx) grid[b.x + dx][b.y] = true;
        }

        int remaining_holes = 0;
        for (int i = 0; i < K; ++i) {
            if (!grid[ab[i].first][ab[i].second]) remaining_holes++;
        }

        vector<Brick> next_bricks = rem_bricks;
        bool failed = false;

        while (remaining_holes > 0) {
            cur_token++;
            priority_queue<tuple<int, int, int>, vector<tuple<int, int, int>>, greater<tuple<int, int, int>>> pq;

            for (int x = 0; x < W; ++x) {
                if (!grid[x][0]) {
                    dist_buf[x][0] = 0; token_buf[x][0] = cur_token;
                    pq.push({0, x, -1});
                }
            }
            for (int x = 0; x < W; ++x) {
                for (int y = 0; y < H; ++y) {
                    if (grid[x][y] && y + 1 < H && !grid[x][y + 1]) {
                        dist_buf[x][y + 1] = 0; token_buf[x][y + 1] = cur_token;
                        pq.push({0, x, y});
                    }
                }
            }

            int target_x = -1, target_y = -1;
            while (!pq.empty()) {
                auto [d, x, y] = pq.top();
                pq.pop();
                if (token_buf[x][y + 1] == cur_token && d > dist_buf[x][y + 1]) continue;

                if (y >= 0 && is_hole[x][y] && !grid[x][y] && d > 0) {
                    target_x = x; target_y = y;
                    break;
                }
                if (y == H - 1) continue;
                int ny = y + 1;

                for (int L : WIDTHS) {
                    int rad = (L - 1) / 2;
                    int bx = x - rad;
                    if (bx < 0 || bx + L > W) continue;

                    bool overlap = false;
                    for (int dx = 0; dx < L; ++dx) {
                        if (grid[bx + dx][ny]) { overlap = true; break; }
                    }
                    if (overlap) continue;

                    int next_dist = d + cost.at(L);
                    for (int dx = 0; dx < L; ++dx) {
                        int nx = bx + dx;
                        if (token_buf[nx][ny + 1] != cur_token || next_dist < dist_buf[nx][ny + 1]) {
                            dist_buf[nx][ny + 1] = next_dist;
                            token_buf[nx][ny + 1] = cur_token;
                            parent_buf[nx][ny + 1] = {x, y, bx, ny, L};
                            pq.push({next_dist, nx, ny});
                        }
                    }
                }
            }

            if (target_x == -1) { failed = true; break; }

            vector<Brick> path_bricks;
            int cx = target_x, cy = target_y;
            while (parent_buf[cx][cy + 1].px != -1 || parent_buf[cx][cy + 1].by != -1) {
                auto p = parent_buf[cx][cy + 1];
                path_bricks.push_back({p.bx, p.by, p.bl});
                if (p.py == -1 || grid[p.px][p.py]) break;
                cx = p.px; cy = p.py;
            }
            reverse(path_bricks.begin(), path_bricks.end());

            for (const auto& b : path_bricks) {
                next_bricks.push_back(b);
                for (int dx = 0; dx < b.l; ++dx) {
                    if (!grid[b.x + dx][b.y]) {
                        grid[b.x + dx][b.y] = true;
                        if (is_hole[b.x + dx][b.y]) remaining_holes--;
                    }
                }
            }
        }

        if (failed) continue;

        prune_solution(next_bricks, ab, cost, W, H, K);
        merge_bricks(next_bricks, ab, cost, W, H, K);
        branch_reconnect(next_bricks, ab, cost, W, H, K);
        prune_solution(next_bricks, ab, cost, W, H, K);

        int next_cost = calc_total_cost(next_bricks);
        if (next_cost < best_cost) {
            best_cost = next_cost;
            best_bricks = next_bricks;
            cur_bricks = next_bricks;
        }
    }

    cur_bricks = best_bricks;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    int W, H, K;
    if (!(cin >> W >> H >> K)) return 0;

    int c1, c3, c5, c7, c9;
    cin >> c1 >> c3 >> c5 >> c7 >> c9;
    map<int, int> cost;
    cost[1] = c1; cost[3] = c3; cost[5] = c5; cost[7] = c7; cost[9] = c9;

    vector<pair<int, int>> ab(K);
    vector<vector<bool>> is_hole(W, vector<bool>(H, false));
    for (int i = 0; i < K; ++i) {
        cin >> ab[i].first >> ab[i].second;
        is_hole[ab[i].first][ab[i].second] = true;
    }

    auto start_time = chrono::steady_clock::now();
    double time_limit_ms = 1650.0;

    auto calc_cost = [&](const vector<Brick>& b_list) {
        int tot = 0;
        for (const auto& b : b_list) tot += cost[b.l];
        return tot;
    };

    // 1. Top-Down による優良初期解のマルチスタート生成（3〜5回）
    vector<Brick> best_ans;
    int min_cost = INF;

    for (int iter = 0; iter < 4; ++iter) {
        int hole_bonus = (iter == 0 ? 4 : rng() % 7);
        double noise = (iter == 0 ? 0.0 : 1.2);
        vector<pair<int, int>> holes = ab;
        if (iter > 0) {
            for (size_t i = 0; i + 1 < holes.size(); ++i) {
                if (holes[i].second == holes[i + 1].second && (rng() % 2 == 0)) {
                    swap(holes[i], holes[i + 1]);
                }
            }
        }

        auto ans = run_top_down_dijkstra(W, H, K, cost, holes, hole_bonus, noise);
        if (is_valid(ans, ab, W, H, K)) {
            int c = calc_cost(ans);
            if (c < min_cost) {
                min_cost = c;
                best_ans = ans;
            }
        }
    }

    // 2. 最良のTop-Down骨格を出発点として、後半みっちり局所探索（再配線）
    local_search_fast(best_ans, ab, cost, is_hole, W, H, K, start_time, 1650.0);

    branch_reconnect(best_ans, ab, cost, W, H, K);
    prune_solution(best_ans, ab, cost, W, H, K);

    cout << best_ans.size() << "\n";
    for (const auto& b : best_ans) {
        cout << b.x << " " << b.y << " " << b.l << "\n";
    }

    return 0;
}
