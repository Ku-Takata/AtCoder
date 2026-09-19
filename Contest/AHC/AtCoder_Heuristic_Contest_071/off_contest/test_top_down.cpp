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
    bool operator==(const Brick& o) const {
        return x == o.x && y == o.y && l == o.l;
    }
};

const int MAX_W = 60;
const int MAX_H = 40;
int dist_buf[MAX_W][MAX_H + 1];
int token_buf[MAX_W][MAX_H + 1];
int cur_token = 0;

struct ParentInfo {
    int px, py, bx, by, bl;
} parent_buf[MAX_W][MAX_H + 1];

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

// 高い穴から順に接続していく Top-Down ダイクストラ
vector<Brick> run_top_down_dijkstra(int W, int H, int K, const map<int, int>& cost, 
                                   vector<pair<int, int>> holes, int hole_bonus) {
    // 高さ y の降順でソート
    sort(holes.begin(), holes.end(), [](const pair<int, int>& a, const pair<int, int>& b) {
        return a.second > b.second;
    });

    vector<vector<bool>> grid(W, vector<bool>(H, false));
    vector<vector<bool>> is_hole_grid(W, vector<bool>(H, false));
    for (const auto& h : holes) is_hole_grid[h.first][h.second] = true;

    vector<Brick> ans;

    for (const auto& target_hole : holes) {
        int thx = target_hole.first, thy = target_hole.second;
        if (grid[thx][thy]) continue; // すでにカバー済み

        // この target_hole に到達する最小コストのパスをダイクストラで探索
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

            // target_hole に到達したか？
            if (y == thy && x == thx && d > 0) {
                reach_x = x; reach_y = y;
                reached = true;
                break;
            }

            if (y >= thy) continue; // ターゲットより上には行かない！
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
                int edge_cost = max(1, cost.at(L) - bonus_count * hole_bonus);
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
    return ans;
}

int main() {
    int W, H, K;
    if (!(cin >> W >> H >> K)) return 0;
    int c1, c3, c5, c7, c9;
    cin >> c1 >> c3 >> c5 >> c7 >> c9;
    map<int, int> cost;
    cost[1] = c1; cost[3] = c3; cost[5] = c5; cost[7] = c7; cost[9] = c9;
    vector<pair<int, int>> ab(K);
    for (int i = 0; i < K; ++i) cin >> ab[i].first >> ab[i].second;

    auto ans = run_top_down_dijkstra(W, H, K, cost, ab, 4);
    int tot = 0;
    for (const auto& b : ans) tot += cost[b.l];
    cerr << "Top-Down Dijkstra Cost: " << tot << ", Bricks: " << ans.size() << endl;

    cout << ans.size() << "\n";
    for (const auto& b : ans) cout << b.x << " " << b.y << " " << b.l << "\n";
    return 0;
}
