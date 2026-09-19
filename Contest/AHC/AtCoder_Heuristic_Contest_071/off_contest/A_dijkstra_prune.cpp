#include <iostream>
#include <vector>
#include <queue>
#include <tuple>
#include <chrono>
#include <algorithm>
#include <map>

using namespace std;

const int INF = 1e9;
const int WIDTHS[5] = {1, 3, 5, 7, 9};

struct Brick {
    int x, y, l;
};

// 案D: 不要レンガの削除（Pruning）
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

                int next_cost = d + cost[L];
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
            bool already_exists = false;
            for (const auto& eb : ans) {
                if (eb.x == b.x && eb.y == b.y && eb.l == b.l) {
                    already_exists = true;
                    break;
                }
            }
            if (!already_exists) {
                ans.push_back(b);
                for (int dx = 0; dx < b.l; ++dx) {
                    if (!grid[b.x + dx][b.y]) {
                        grid[b.x + dx][b.y] = true;
                        if (is_hole[b.x + dx][b.y]) {
                            remaining_holes--;
                        }
                    }
                }
            }
        }
    }

    // 案D: 不要レンガの削除
    prune_solution(ans, ab, W, H, K);

    // 解の出力
    cout << ans.size() << "\n";
    for (const auto& b : ans) {
        cout << b.x << " " << b.y << " " << b.l << "\n";
    }

    return 0;
}
