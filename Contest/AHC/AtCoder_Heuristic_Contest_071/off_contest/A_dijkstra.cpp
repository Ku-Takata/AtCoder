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

int main() {
    // 高速入出力
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

    // grid[x][y]: マス(x, y)が既にレンガで覆われているか
    vector<vector<bool>> grid(W, vector<bool>(H, false));
    vector<Brick> ans;
    int remaining_holes = K;

    #ifdef DEBUG
    cerr << "[DEBUG] W=" << W << ", H=" << H << ", K=" << K << endl;
    cerr << "[DEBUG] Costs: c1=" << c1 << ", c3=" << c3 << ", c5=" << c5 << ", c7=" << c7 << ", c9=" << c9 << endl;
    #endif

    int step = 0;
    while (remaining_holes > 0) {
        step++;

        // 多始点ダイクストラ法
        // 状態: 支持点 (x, y) ※ y = -1 は床(Ground)を表す
        // pqの要素: (累計コスト, x, y)
        priority_queue<tuple<int, int, int>, vector<tuple<int, int, int>>, greater<tuple<int, int, int>>> pq;
        
        // dist[x][y + 1]: 状態 (x, y) への最小コスト
        vector<vector<int>> dist(W, vector<int>(H + 1, INF));

        // 経路復元用親ノード情報: (直前の支持点px, 直前の支持点py, 配置したレンガbx, by, bl)
        struct ParentInfo {
            int px, py, bx, by, bl;
        };
        vector<vector<ParentInfo>> parent(W, vector<ParentInfo>(H + 1, {-1, -1, -1, -1, -1}));

        // 始点1: 床 (y = -1) からの支持。y=0 にレンガが未配置の列はコスト0で開始可能
        for (int x = 0; x < W; ++x) {
            if (!grid[x][0]) {
                dist[x][0] = 0;
                pq.push({0, x, -1});
            }
        }

        // 始点2: 既に木に組み込まれた全レンガのマス (x, y)。真上が空いていればコスト0で開始可能
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

            // 未回収の穴マスに到達した場合（新しくレンガを置いた場合のみ）
            if (y >= 0 && is_hole[x][y] && !grid[x][y] && d > 0) {
                target_x = x;
                target_y = y;
                break;
            }

            if (y == H - 1) continue;

            int ny = y + 1;
            // 支持点 (x, y) の直上 ny 段目に、中心を x とするレンガを配置
            for (int L : WIDTHS) {
                int rad = (L - 1) / 2;
                int bx = x - rad;
                if (bx < 0 || bx + L > W) continue; // 壁外判定

                // 既存レンガとの重複チェック
                bool overlap = false;
                for (int dx = 0; dx < L; ++dx) {
                    if (grid[bx + dx][ny]) {
                        overlap = true;
                        break;
                    }
                }
                if (overlap) continue;

                int next_cost = d + cost[L];
                // 配置したレンガが覆う全マス nx が、次の段への支持点となる
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

        if (target_x == -1) {
            #ifdef DEBUG
            cerr << "[DEBUG] Warning: 到達可能な未回収の穴が見つかりません。残り穴数: " << remaining_holes << endl;
            #endif
            break;
        }

        // 最短パス上のレンガを復元
        vector<Brick> path_bricks;
        int cx = target_x, cy = target_y;
        while (parent[cx][cy + 1].px != -1 || parent[cx][cy + 1].by != -1) {
            auto p = parent[cx][cy + 1];
            path_bricks.push_back({p.bx, p.by, p.bl});
            // 既存の木、または床に衝突したらバックトラック終了
            if (p.py == -1 || grid[p.px][p.py]) {
                break;
            }
            cx = p.px;
            cy = p.py;
        }

        reverse(path_bricks.begin(), path_bricks.end());

        int newly_covered = 0;
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
                            newly_covered++;
                        }
                    }
                }
            }
        }

        #ifdef DEBUG
        cerr << "[DEBUG] Step " << step << ": 穴(" << target_x << ", " << target_y 
             << ")を接続. 追加レンガ数: " << path_bricks.size() 
             << ", 今回覆った穴数: " << newly_covered 
             << ", 残り穴数: " << remaining_holes << endl;
        #endif
    }

    // 解の出力
    cout << ans.size() << "\n";
    for (const auto& b : ans) {
        cout << b.x << " " << b.y << " " << b.l << "\n";
    }

    auto end_time = chrono::steady_clock::now();
    double elapsed_ms = chrono::duration<double, milli>(end_time - start_time).count();

    #ifdef DEBUG
    long long total_c = 0;
    for (const auto& b : ans) total_c += cost[b.l];
    long long score = max(0LL, (long long)W * H * c1 - total_c + 1);
    cerr << "[DEBUG] 完了. レンガ個数: " << ans.size() 
         << ", 合計費用: " << total_c 
         << ", スコア: " << score 
         << ", 実行時間: " << elapsed_ms << " ms" << endl;
    #endif

    return 0;
}