#include <iostream>
#include <vector>
#include <queue>
#include <tuple>
#include <chrono>
#include <algorithm>
#include <map>
#include <cmath>
#include <random>

using namespace std;

const int INF = 1e9;
const int WIDTHS[5] = {1, 3, 5, 7, 9};

struct Brick {
    int x, y, l;
};

mt19937 rng(42);

// 案D: 不要レンガの削除（Pruning / 枝刈り）
void prune_solution(vector<Brick>& bricks, const vector<pair<int, int>>& ab, int W, int H, int K) {
    int n = bricks.size();
    vector<bool> alive(n, true);

    vector<vector<int>> cell_brick(W, vector<int>(H, -1));
    for (int i = 0; i < n; ++i) {
        for (int dx = 0; dx < bricks[i].l; ++dx) {
            cell_brick[bricks[i].x + dx][bricks[i].y] = i;
        }
    }

    vector<int> parent_id(n, -1);
    vector<int> child_count(n, 0);

    for (int i = 0; i < n; ++i) {
        if (bricks[i].y > 0) {
            int mid_x = bricks[i].x + (bricks[i].l - 1) / 2;
            int p = cell_brick[mid_x][bricks[i].y - 1];
            parent_id[i] = p;
            if (p != -1) {
                child_count[p]++;
            }
        }
    }

    vector<int> hole_cover(K, 0);
    vector<vector<int>> brick_holes(n);
    for (int h_idx = 0; h_idx < K; ++h_idx) {
        int hx = ab[h_idx].first;
        int hy = ab[h_idx].second;
        int b_id = cell_brick[hx][hy];
        if (b_id != -1) {
            hole_cover[h_idx]++;
            brick_holes[b_id].push_back(h_idx);
        }
    }

    queue<int> q;
    for (int i = 0; i < n; ++i) {
        if (child_count[i] == 0) {
            q.push(i);
        }
    }

    while (!q.empty()) {
        int u = q.front();
        q.pop();

        if (!alive[u]) continue;
        if (child_count[u] > 0) continue;

        bool covers_critical_hole = false;
        for (int h_idx : brick_holes[u]) {
            if (hole_cover[h_idx] <= 1) {
                covers_critical_hole = true;
                break;
            }
        }

        if (!covers_critical_hole) {
            alive[u] = false;
            for (int h_idx : brick_holes[u]) {
                hole_cover[h_idx]--;
            }
            int p = parent_id[u];
            if (p != -1 && alive[p]) {
                child_count[p]--;
                if (child_count[p] == 0) {
                    q.push(p);
                }
            }
        }
    }

    vector<Brick> res;
    for (int i = 0; i < n; ++i) {
        if (alive[i]) {
            res.push_back(bricks[i]);
        }
    }
    bricks = res;
}

// 解の妥当性検証
bool is_valid_solution(const vector<Brick>& bricks, const vector<pair<int, int>>& ab, int W, int H, int K) {
    if (bricks.empty()) return false;
    vector<vector<int>> grid(W, vector<int>(H, -1));

    // 壁内 & 重複チェック
    for (size_t i = 0; i < bricks.size(); ++i) {
        const auto& b = bricks[i];
        if (b.l != 1 && b.l != 3 && b.l != 5 && b.l != 7 && b.l != 9) return false;
        if (b.x < 0 || b.x + b.l > W || b.y < 0 || b.y >= H) return false;
        for (int dx = 0; dx < b.l; ++dx) {
            if (grid[b.x + dx][b.y] != -1) return false; // 重複
            grid[b.x + dx][b.y] = i;
        }
    }

    // 支持条件チェック
    for (const auto& b : bricks) {
        if (b.y > 0) {
            int mid_x = b.x + (b.l - 1) / 2;
            if (grid[mid_x][b.y - 1] == -1) return false; // 支持なし
        }
    }

    // 穴の全カバーチェック
    for (int i = 0; i < K; ++i) {
        if (grid[ab[i].first][ab[i].second] == -1) return false;
    }

    return true;
}

// 案A: 柱+アームの解を生成する関数
vector<Brick> solve_pillars(int W, int H, int K, const map<int, int>& cost, 
                           const vector<pair<int, int>>& ab, 
                           const vector<vector<int>>& holes_at_y, 
                           const vector<vector<bool>>& is_hole,
                           chrono::steady_clock::time_point start_time, double time_limit_ms) {
    
    auto get_min_L = [](int rad) {
        if (rad <= 0) return 1;
        if (rad <= 1) return 3;
        if (rad <= 2) return 5;
        if (rad <= 3) return 7;
        return 9;
    };

    auto evaluate_pillars = [&](const vector<int>& P, vector<Brick>& out_bricks) -> pair<int, int> {
        out_bricks.clear();
        int m = P.size();
        if (m == 0) return {INF, K};

        // 各柱の初期幅は 1
        vector<vector<int>> brick_l(m, vector<int>(H, 1));
        vector<int> max_y(m, -1);
        int uncovered = 0;

        for (int y = 0; y < H; ++y) {
            for (int hx : holes_at_y[y]) {
                int best_i = -1;
                int min_inc_cost = INF;

                for (int i = 0; i < m; ++i) {
                    int dist = abs(hx - P[i]);
                    if (dist > 4) continue;

                    int cur_rad = (brick_l[i][y] - 1) / 2;
                    int new_rad = max(cur_rad, dist);
                    int new_L = get_min_L(new_rad);

                    int bx = P[i] - (new_L - 1) / 2;
                    int rx = bx + new_L - 1;
                    if (bx < 0 || rx >= W) continue;

                    // 隣接する柱との厳密な重複チェック
                    bool conflict = false;
                    if (i > 0) {
                        int r_prev = P[i - 1] + (brick_l[i - 1][y] - 1) / 2;
                        if (r_prev >= bx) conflict = true;
                    }
                    if (i + 1 < m) {
                        int l_next = P[i + 1] - (brick_l[i + 1][y] - 1) / 2;
                        if (rx >= l_next) conflict = true;
                    }
                    if (conflict) continue;

                    int inc_cost = cost.at(new_L) - cost.at(brick_l[i][y]);
                    if (inc_cost < min_inc_cost) {
                        min_inc_cost = inc_cost;
                        best_i = i;
                    }
                }

                if (best_i != -1) {
                    int dist = abs(hx - P[best_i]);
                    int cur_rad = (brick_l[best_i][y] - 1) / 2;
                    int new_rad = max(cur_rad, dist);
                    brick_l[best_i][y] = get_min_L(new_rad);
                    max_y[best_i] = max(max_y[best_i], y);
                } else {
                    uncovered++;
                }
            }
        }

        int total_cost = 0;
        for (int i = 0; i < m; ++i) {
            for (int y = 0; y <= max_y[i]; ++y) {
                int L = brick_l[i][y];
                int bx = P[i] - (L - 1) / 2;
                out_bricks.push_back({bx, y, L});
                total_cost += cost.at(L);
            }
        }

        return {total_cost, uncovered};
    };

    vector<int> best_P;
    for (int x = 4; x < W; x += 8) best_P.push_back(x);
    if (best_P.back() < W - 4) best_P.push_back(W - 1 - 4);
    sort(best_P.begin(), best_P.end());
    best_P.erase(unique(best_P.begin(), best_P.end()), best_P.end());

    vector<Brick> best_bricks;
    auto [best_eval_cost, best_uncovered] = evaluate_pillars(best_P, best_bricks);
    vector<int> cur_P = best_P;

    int iter = 0;
    while (true) {
        if ((iter & 255) == 0) {
            auto now = chrono::steady_clock::now();
            double el = chrono::duration<double, milli>(now - start_time).count();
            if (el > time_limit_ms) break;
        }
        iter++;

        vector<int> next_P = cur_P;
        int type = rng() % 3;

        if (type == 0 && !next_P.empty()) {
            int idx = rng() % next_P.size();
            int shift = (rng() % 2 == 0 ? 1 : -1) * (1 + rng() % 2);
            int nx = next_P[idx] + shift;
            if (nx >= 0 && nx < W) next_P[idx] = nx;
        } else if (type == 1 && next_P.size() < 10) {
            next_P.push_back(rng() % W);
        } else if (type == 2 && next_P.size() > 4) {
            int idx = rng() % next_P.size();
            next_P.erase(next_P.begin() + idx);
        }

        sort(next_P.begin(), next_P.end());
        next_P.erase(unique(next_P.begin(), next_P.end()), next_P.end());

        bool too_close = false;
        for (size_t i = 0; i + 1 < next_P.size(); ++i) {
            if (next_P[i + 1] - next_P[i] < 2) {
                too_close = true;
                break;
            }
        }
        if (too_close) continue;

        vector<Brick> temp_bricks;
        auto [eval_cost, unc] = evaluate_pillars(next_P, temp_bricks);

        long long current_score = (long long)best_eval_cost + (long long)best_uncovered * 1000;
        long long next_score = (long long)eval_cost + (long long)unc * 1000;

        if (next_score < current_score) {
            best_eval_cost = eval_cost;
            best_uncovered = unc;
            best_P = next_P;
            cur_P = next_P;
            best_bricks = temp_bricks;
        } else if (rng() % 100 < 5) {
            cur_P = next_P;
        }
    }

    // 未回収の穴をダイクストラで回収
    vector<vector<bool>> grid(W, vector<bool>(H, false));
    for (const auto& b : best_bricks) {
        for (int dx = 0; dx < b.l; ++dx) {
            grid[b.x + dx][b.y] = true;
        }
    }

    int remaining_holes = 0;
    for (int i = 0; i < K; ++i) {
        if (!grid[ab[i].first][ab[i].second]) remaining_holes++;
    }

    vector<Brick> ans = best_bricks;

    while (remaining_holes > 0) {
        priority_queue<tuple<int, int, int>, vector<tuple<int, int, int>>, greater<tuple<int, int, int>>> pq;
        vector<vector<int>> dist(W, vector<int>(H + 1, INF));
        struct ParentInfo {
            int px, py, bx, by, bl;
        };
        vector<vector<ParentInfo>> parent(W, vector<ParentInfo>(H + 1, {-1, -1, -1, -1, -1}));

        for (int x = 0; x < W; ++x) {
            if (!grid[x][0]) {
                dist[x][0] = 0;
                pq.push({0, x, -1});
            }
        }
        for (int x = 0; x < W; ++x) {
            for (int y = 0; y < H; ++y) {
                if (grid[x][y]) {
                    if (y + 1 < H && !grid[x][y + 1]) {
                        dist[x][y + 1] = 0;
                        pq.push({0, x, y});
                    }
                }
            }
        }

        int target_x = -1, target_y = -1;
        while (!pq.empty()) {
            auto [d, x, y] = pq.top();
            pq.pop();

            if (d > dist[x][y + 1]) continue;

            if (y >= 0 && is_hole[x][y] && !grid[x][y] && d > 0) {
                target_x = x;
                target_y = y;
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
                    if (grid[bx + dx][ny]) {
                        overlap = true;
                        break;
                    }
                }
                if (overlap) continue;

                int next_cost = d + cost.at(L);
                for (int dx = 0; dx < L; ++dx) {
                    int nx = bx + dx;
                    if (next_cost < dist[nx][ny + 1]) {
                        dist[nx][ny + 1] = next_cost;
                        parent[nx][ny + 1] = {x, y, bx, ny, L};
                        pq.push({next_cost, nx, ny});
                    }
                }
            }
        }

        if (target_x == -1) break;

        vector<Brick> path_bricks;
        int cx = target_x, cy = target_y;
        while (parent[cx][cy + 1].px != -1 || parent[cx][cy + 1].by != -1) {
            auto p = parent[cx][cy + 1];
            path_bricks.push_back({p.bx, p.by, p.bl});
            if (p.py == -1 || grid[p.px][p.py]) break;
            cx = p.px;
            cy = p.py;
        }
        reverse(path_bricks.begin(), path_bricks.end());

        for (const auto& b : path_bricks) {
            ans.push_back(b);
            for (int dx = 0; dx < b.l; ++dx) {
                if (!grid[b.x + dx][b.y]) {
                    grid[b.x + dx][b.y] = true;
                    if (is_hole[b.x + dx][b.y]) remaining_holes--;
                }
            }
        }
    }

    prune_solution(ans, ab, W, H, K);
    return ans;
}

// ダイクストラ法による解生成
vector<Brick> solve_dijkstra(int W, int H, int K, const map<int, int>& cost, 
                             const vector<pair<int, int>>& ab, 
                             const vector<vector<bool>>& is_hole) {
    vector<vector<bool>> grid(W, vector<bool>(H, false));
    vector<Brick> ans;
    int remaining_holes = K;

    while (remaining_holes > 0) {
        priority_queue<tuple<int, int, int>, vector<tuple<int, int, int>>, greater<tuple<int, int, int>>> pq;
        vector<vector<int>> dist(W, vector<int>(H + 1, INF));
        struct ParentInfo {
            int px, py, bx, by, bl;
        };
        vector<vector<ParentInfo>> parent(W, vector<ParentInfo>(H + 1, {-1, -1, -1, -1, -1}));

        for (int x = 0; x < W; ++x) {
            if (!grid[x][0]) {
                dist[x][0] = 0;
                pq.push({0, x, -1});
            }
        }
        for (int x = 0; x < W; ++x) {
            for (int y = 0; y < H; ++y) {
                if (grid[x][y]) {
                    if (y + 1 < H && !grid[x][y + 1]) {
                        dist[x][y + 1] = 0;
                        pq.push({0, x, y});
                    }
                }
            }
        }

        int target_x = -1, target_y = -1;
        while (!pq.empty()) {
            auto [d, x, y] = pq.top();
            pq.pop();

            if (d > dist[x][y + 1]) continue;

            if (y >= 0 && is_hole[x][y] && !grid[x][y] && d > 0) {
                target_x = x;
                target_y = y;
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
                    if (grid[bx + dx][ny]) {
                        overlap = true;
                        break;
                    }
                }
                if (overlap) continue;

                int next_cost = d + cost.at(L);
                for (int dx = 0; dx < L; ++dx) {
                    int nx = bx + dx;
                    if (next_cost < dist[nx][ny + 1]) {
                        dist[nx][ny + 1] = next_cost;
                        parent[nx][ny + 1] = {x, y, bx, ny, L};
                        pq.push({next_cost, nx, ny});
                    }
                }
            }
        }

        if (target_x == -1) break;

        vector<Brick> path_bricks;
        int cx = target_x, cy = target_y;
        while (parent[cx][cy + 1].px != -1 || parent[cx][cy + 1].by != -1) {
            auto p = parent[cx][cy + 1];
            path_bricks.push_back({p.bx, p.by, p.bl});
            if (p.py == -1 || grid[p.px][p.py]) break;
            cx = p.px;
            cy = p.py;
        }
        reverse(path_bricks.begin(), path_bricks.end());

        for (const auto& b : path_bricks) {
            bool already = false;
            for (const auto& eb : ans) {
                if (eb.x == b.x && eb.y == b.y && eb.l == b.l) {
                    already = true;
                    break;
                }
            }
            if (!already) {
                ans.push_back(b);
                for (int dx = 0; dx < b.l; ++dx) {
                    if (!grid[b.x + dx][b.y]) {
                        grid[b.x + dx][b.y] = true;
                        if (is_hole[b.x + dx][b.y]) remaining_holes--;
                    }
                }
            }
        }
    }

    prune_solution(ans, ab, W, H, K);
    return ans;
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
    vector<vector<int>> holes_at_y(H);
    for (int i = 0; i < K; ++i) {
        cin >> ab[i].first >> ab[i].second;
        is_hole[ab[i].first][ab[i].second] = true;
        holes_at_y[ab[i].second].push_back(ab[i].first);
    }

    auto start_time = chrono::steady_clock::now();

    // 案A（柱+アーム+Pruning）を実行
    vector<Brick> ans_pillar = solve_pillars(W, H, K, cost, ab, holes_at_y, is_hole, start_time, 800.0);

    // ダイクストラ+Pruning を実行
    vector<Brick> ans_dijkstra = solve_dijkstra(W, H, K, cost, ab, is_hole);

    // コスト計算
    auto calc_cost = [&](const vector<Brick>& b_list) {
        int tot = 0;
        for (const auto& b : b_list) tot += cost[b.l];
        return tot;
    };

    bool pillar_valid = is_valid_solution(ans_pillar, ab, W, H, K);
    bool dijkstra_valid = is_valid_solution(ans_dijkstra, ab, W, H, K);

    int cost_pillar = (pillar_valid ? calc_cost(ans_pillar) : INF);
    int cost_dijkstra = (dijkstra_valid ? calc_cost(ans_dijkstra) : INF);

    // より良い方を安全に採用
    vector<Brick> best_ans;
    if (pillar_valid && cost_pillar < cost_dijkstra) {
        best_ans = ans_pillar;
    } else if (dijkstra_valid) {
        best_ans = ans_dijkstra;
    } else {
        best_ans = ans_pillar; // フォールバック
    }

    // 解の出力
    cout << best_ans.size() << "\n";
    for (const auto& b : best_ans) {
        cout << b.x << " " << b.y << " " << b.l << "\n";
    }

    return 0;
}
