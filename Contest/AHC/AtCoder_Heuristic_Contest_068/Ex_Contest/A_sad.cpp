#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <random>
#include <chrono>

using namespace std;

const int N = 20;
int initial_grid[N][N];
int grid[N][N];
string V_walls[N];
string H_walls[N - 1];

int dist_tbl[N * N][N * N];

struct Operation {
    char d;
    int r, c, h, w;
};

inline double get_time() {
    static auto start = chrono::system_clock::now();
    auto now = chrono::system_clock::now();
    return chrono::duration_cast<chrono::duration<double>>(now - start).count();
}

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

bool is_valid_rect(int r, int c, int h, int w) {
    for (int i = 0; i < h; ++i) {
        for (int j = 0; j < w; ++j) {
            if (j + 1 < w && !can_move(r + i, c + j, r + i, c + j + 1)) return false;
            if (i + 1 < h && !can_move(r + i, c + j, r + i + 1, c + j)) return false;
        }
    }
    return true;
}

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

inline int calc_cost(int u, int val) {
    int d = dist_tbl[u][val];
    return d * d;
}

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
                diff -= calc_cost(u1, val1) + calc_cost(u2, val2);
                diff += calc_cost(u1, val2) + calc_cost(u2, val1);
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
                diff -= calc_cost(u1, val1) + calc_cost(u2, val2);
                diff += calc_cost(u1, val2) + calc_cost(u2, val1);
            }
        }
    }
    return diff;
}

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

    get_time();

    int dummy_N;
    if (!(cin >> dummy_N)) return 0;

    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) {
            cin >> initial_grid[i][j];
        }
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

    vector<Operation> ops_area_8;
    vector<Operation> ops_area_4;
    vector<Operation> ops_area_2;
    vector<Operation> ops_area_1;

    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int h = 1; r + h <= N; ++h) {
                for (int w = 1; c + w <= N; ++w) {
                    if (is_valid_rect(r, c, h, w)) {
                        int area = h * w;
                        Operation v_op = {'V', r, c, h, w};
                        Operation h_op = {'H', r, c, h, w};

                        auto add_op = [&](const Operation& op) {
                            if (area >= 8) ops_area_8.push_back(op);
                            else if (area >= 4) ops_area_4.push_back(op);
                            else if (area >= 2) ops_area_2.push_back(op);
                            else ops_area_1.push_back(op);
                        };

                        if (h % 2 == 0) add_op(v_op);
                        if (w % 2 == 0) add_op(h_op);
                    }
                }
            }
        }
    }
    
    cerr << "[Time] Initialization done: " << get_time() << " s\n";

    mt19937 rng(42);

    // --- Phase 1: 1.5秒まで何度もリスタートして最良の盤面を探す ---
    int best_p1_cost = 2e9;
    vector<Operation> best_p1_ops;
    int best_p1_grid[N][N];

    const double P1_TIME_LIMIT = 1.5;
    int p1_iterations = 0;

    while (get_time() < P1_TIME_LIMIT) {
        p1_iterations++;
        for (int i = 0; i < N; ++i) {
            for (int j = 0; j < N; ++j) grid[i][j] = initial_grid[i][j];
        }

        vector<Operation> current_ops;
        
        int step = 0;
        int no_improve = 0;
        const int MAX_STEPS = 2000;
        
        // 改善が見込めなくなったら（局所解に陥ったら）早めに打ち切る
        while (step < MAX_STEPS && no_improve < 250 && current_ops.size() < 30000) {
            double progress = (double)step / MAX_STEPS;
            
            vector<Operation>* target_list = &ops_area_1;
            if (progress < 0.25 && !ops_area_8.empty()) target_list = &ops_area_8;
            else if (progress < 0.50 && !ops_area_4.empty()) target_list = &ops_area_4;
            else if (progress < 0.75 && !ops_area_2.empty()) target_list = &ops_area_2;

            if (target_list->empty()) target_list = &ops_area_1;

            int best_diff = 0;
            Operation best_op;
            bool found = false;

            int samples = min(200, (int)target_list->size());
            for (int i = 0; i < samples; ++i) {
                const auto& op = (*target_list)[rng() % target_list->size()];
                int diff = eval_diff(op);
                if (diff < best_diff) {
                    best_diff = diff;
                    best_op = op;
                    found = true;
                }
            }

            if (found) {
                apply_op(best_op);
                current_ops.push_back(best_op);
                no_improve = 0; // 改善したためリセット
            } else {
                no_improve++; // 改善しなかった
            }
            step++;
        }

        int current_cost = 0;
        for (int r = 0; r < N; ++r) {
            for (int c = 0; c < N; ++c) {
                current_cost += calc_cost(r * N + c, grid[r][c]);
            }
        }

        if (current_cost < best_p1_cost) {
            best_p1_cost = current_cost;
            best_p1_ops = current_ops;
            for (int r = 0; r < N; ++r) {
                for (int c = 0; c < N; ++c) best_p1_grid[r][c] = grid[r][c];
            }
        }
    }
    
    cerr << "[Time] Phase 1 done: " << get_time() << " s\n";
    cerr << "[Debug] Phase 1 iterations: " << p1_iterations << "\n";

    // --- Phase 2: 選ばれた盤面に対して細かい微調整をマルチスタート ---
    // 残りの時間(1.5秒〜1.8秒)を使用
    int best_p2_ops_count = 1e9;
    vector<Operation> best_phase2_ops;
    const double TIME_LIMIT = 1.8; 
    int p2_iterations = 0;

    while (get_time() < TIME_LIMIT) {
        p2_iterations++;
        for (int r = 0; r < N; ++r) {
            for (int c = 0; c < N; ++c) grid[r][c] = best_p1_grid[r][c];
        }

        vector<int> parent(N * N, -1);
        vector<int> order;
        vector<bool> visited(N * N, false);
        queue<int> q;
        
        int root = rng() % (N * N);
        q.push(root);
        visited[root] = true;
        
        while (!q.empty()) {
            int u = q.front();
            q.pop();
            order.push_back(u);
            
            vector<int> nxt = adj[u];
            shuffle(nxt.begin(), nxt.end(), rng); 
            
            for (int v : nxt) {
                if (!visited[v]) {
                    visited[v] = true;
                    parent[v] = u;
                    q.push(v);
                }
            }
        }
        reverse(order.begin(), order.end());
        vector<bool> fixed(N * N, false);
        vector<Operation> current_p2_ops;

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
                current_p2_ops.push_back(op);
            }
        }

        if (current_p2_ops.size() < best_p2_ops_count) {
            best_p2_ops_count = current_p2_ops.size();
            best_phase2_ops = current_p2_ops;
        }
    }
    
    cerr << "[Time] Phase 2 done: " << get_time() << " s\n";
    cerr << "[Debug] Phase 2 iterations: " << p2_iterations << "\n";
    cerr << "[Debug] Total operations: " << best_p1_ops.size() + best_phase2_ops.size() << "\n";

    // 結果出力
    for (const auto& op : best_p1_ops) {
        cout << op.d << " " << op.r << " " << op.c << " " << op.h << " " << op.w << "\n";
    }
    for (const auto& op : best_phase2_ops) {
        cout << op.d << " " << op.r << " " << op.c << " " << op.h << " " << op.w << "\n";
    }

    return 0;
}