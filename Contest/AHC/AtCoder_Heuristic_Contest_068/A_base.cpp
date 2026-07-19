#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <random>

using namespace std;

const int N = 20;
int grid[N][N];
string V_walls[N];
string H_walls[N - 1];

int dist_tbl[N * N][N * N];

struct Operation {
    char d;
    int r, c, h, w;
};

// 2つの隣接マスの間に壁がないか判定
bool can_move(int r1, int c1, int r2, int c2) {
    if (r1 < 0 || r1 >= N || c1 < 0 || c1 >= N) return false;
    if (r2 < 0 || r2 >= N || c2 < 0 || c2 >= N) return false;
    if (r1 == r2) {
        if (c1 > c2) swap(c1, c2);
        if (V_walls[r1][c1] == '1') return false;
    } else {
        if (r1 > r2) swap(r1, r2);
        if (H_walls[r1][c1] == '1') return false;
    }
    return true;
}

// 矩形内に壁がないか確認
bool is_valid_rect(int r, int c, int h, int w) {
    for (int i = 0; i < h; ++i) {
        for (int j = 0; j < w; ++j) {
            if (j + 1 < w && !can_move(r + i, c + j, r + i, c + j + 1)) return false;
            if (i + 1 < h && !can_move(r + i, c + j, r + i + 1, c + j)) return false;
        }
    }
    return true;
}

// 全頂点対の最短距離を事前計算
void precompute_distances(const vector<vector<int>>& adj) {
    for (int i = 0; i < N * N; ++i) {
        for (int j = 0; j < N * N; ++j) dist_tbl[i][j] = 1e9;
        dist_tbl[i][i] = 0;
        queue<int> q;
        q.push(i);
        while (!q.empty()) {
            int u = q.front();
            q.pop();
            for (int v : adj[u]) {
                if (dist_tbl[i][v] > dist_tbl[i][u] + 1) {
                    dist_tbl[i][v] = dist_tbl[i][u] + 1;
                    q.push(v);
                }
            }
        }
    }
}

// 操作による距離差分を計算（負なら改善）
int eval_diff(const Operation& op) {
    int diff = 0;
    if (op.d == 'V') {
        int half = op.h / 2;
        for (int x = 0; x < half; ++x) {
            for (int y = 0; y < op.w; ++y) {
                int u1 = (op.r + x) * N + (op.c + y);
                int u2 = (op.r + half + x) * N + (op.c + y);
                int val1 = grid[op.r + x][op.c + y];
                int val2 = grid[op.r + half + x][op.c + y];
                diff -= dist_tbl[u1][val1] + dist_tbl[u2][val2];
                diff += dist_tbl[u1][val2] + dist_tbl[u2][val1];
            }
        }
    } else {
        int half = op.w / 2;
        for (int x = 0; x < op.h; ++x) {
            for (int y = 0; y < half; ++y) {
                int u1 = (op.r + x) * N + (op.c + y);
                int u2 = (op.r + x) * N + (op.c + half + y);
                int val1 = grid[op.r + x][op.c + y];
                int val2 = grid[op.r + x][op.c + half + y];
                diff -= dist_tbl[u1][val1] + dist_tbl[u2][val2];
                diff += dist_tbl[u1][val2] + dist_tbl[u2][val1];
            }
        }
    }
    return diff;
}

// 実際にスワップを適用
void apply_op(const Operation& op) {
    if (op.d == 'V') {
        int half = op.h / 2;
        for (int x = 0; x < half; ++x) {
            for (int y = 0; y < op.w; ++y) {
                swap(grid[op.r + x][op.c + y], grid[op.r + half + x][op.c + y]);
            }
        }
    } else {
        int half = op.w / 2;
        for (int x = 0; x < op.h; ++x) {
            for (int y = 0; y < half; ++y) {
                swap(grid[op.r + x][op.c + y], grid[op.r + x][op.c + half + y]);
            }
        }
    }
}

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);

    int dummy_N;
    if (!(cin >> dummy_N)) return 0;

    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) cin >> grid[i][j];
    }
    for (int i = 0; i < N; ++i) cin >> V_walls[i];
    for (int i = 0; i < N - 1; ++i) cin >> H_walls[i];

    vector<vector<int>> adj(N * N);
    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            int u = r * N + c;
            if (can_move(r, c, r, c + 1)) {
                adj[u].push_back(u + 1);
                adj[u + 1].push_back(u);
            }
            if (can_move(r, c, r + 1, c)) {
                adj[u].push_back(u + N);
                adj[u + N].push_back(u);
            }
        }
    }
    precompute_distances(adj);

    // 可能なすべての操作を事前列挙（壁のない矩形）
    vector<Operation> all_possible_ops;
    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int h = 1; r + h <= N; ++h) {
                for (int w = 1; c + w <= N; ++w) {
                    if (is_valid_rect(r, c, h, w)) {
                        if (h % 2 == 0) all_possible_ops.push_back({'V', r, c, h, w});
                        if (w % 2 == 0) all_possible_ops.push_back({'H', r, c, h, w});
                    }
                }
            }
        }
    }

    mt19937 rng(42);
    vector<Operation> ops;
    
    // 【フェーズ1】 大小の長方形を使って距離を貪欲に減らす
    int no_improve_count = 0;
    while (no_improve_count < 2000 && ops.size() < 30000) {
        int best_diff = 0;
        Operation best_op;
        bool found = false;

        // 毎回ランダムにいくつかの操作を試す (全探索すると重いためサンプリング)
        int samples = min(200, (int)all_possible_ops.size());
        for (int i = 0; i < samples; ++i) {
            const auto& op = all_possible_ops[rng() % all_possible_ops.size()];
            int diff = eval_diff(op);
            if (diff < best_diff) {
                best_diff = diff;
                best_op = op;
                found = true;
            }
        }

        if (found) {
            apply_op(best_op);
            ops.push_back(best_op);
            no_improve_count = 0;
        } else {
            no_improve_count++;
        }
    }

    // 【フェーズ2】 残りの微調整（全域木・葉から確定）
    vector<int> parent(N * N, -1);
    vector<int> order;
    vector<bool> visited(N * N, false);
    queue<int> q;
    q.push(0);
    visited[0] = true;
    while (!q.empty()) {
        int u = q.front();
        q.pop();
        order.push_back(u);
        for (int v : adj[u]) {
            if (!visited[v]) {
                visited[v] = true;
                parent[v] = u;
                q.push(v);
            }
        }
    }
    reverse(order.begin(), order.end());
    vector<bool> fixed(N * N, false);

    for (int target_pos : order) {
        fixed[target_pos] = true;
        int target_val = target_pos;
        int current_pos = -1;
        for (int r = 0; r < N; ++r) {
            for (int c = 0; c < N; ++c) {
                if (grid[r][c] == target_val) {
                    current_pos = r * N + c;
                }
            }
        }
        if (current_pos == target_pos) continue;

        vector<int> p(N * N, -1);
        vector<bool> vis(N * N, false);
        queue<int> path_q;
        path_q.push(current_pos);
        vis[current_pos] = true;

        while (!path_q.empty()) {
            int u = path_q.front();
            path_q.pop();
            if (u == target_pos) break;

            for (int v : adj[u]) {
                if (!vis[v] && (!fixed[v] || v == target_pos)) {
                    vis[v] = true;
                    p[v] = u;
                    path_q.push(v);
                }
            }
        }

        vector<int> path;
        int curr = target_pos;
        while (curr != current_pos) {
            path.push_back(curr);
            curr = p[curr];
        }
        path.push_back(current_pos);
        reverse(path.begin(), path.end());

        for (size_t i = 0; i < path.size() - 1; ++i) {
            int u = path[i];
            int v = path[i + 1];
            int r1 = u / N, c1 = u % N;
            int r2 = v / N, c2 = v % N;
            Operation op;
            if (r1 == r2) {
                op = {'H', r1, min(c1, c2), 1, 2};
            } else {
                op = {'V', min(r1, r2), c1, 2, 1};
            }
            apply_op(op);
            ops.push_back(op);
        }
    }

    // 結果出力
    for (const auto& op : ops) {
        cout << op.d << " " << op.r << " " << op.c << " " << op.h << " " << op.w << "\n";
    }

    return 0;
}