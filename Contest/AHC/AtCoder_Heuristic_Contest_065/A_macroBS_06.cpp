#include <iostream>
#include <vector>
#include <queue>
#include <algorithm>
#include <chrono>

using namespace std;
using namespace std::chrono;

// 高速乱数生成器
uint32_t xor128() {
    static uint32_t x = 123456789, y = 362436069, z = 521288629, w = 88675123;
    uint32_t t = x ^ (x << 11);
    x = y; y = z; z = w;
    return w = (w ^ (w >> 19)) ^ (t ^ (t >> 8));
}

int init_grid[20][20];
int init_pos_r[400];
int init_pos_c[400];

int belt_r[20][40];
int belt_c[20][40];

int cb_m[20][20][2];
int cb_idx[20][20][2];
int cb_cnt[20][20];
int dist_to_exit[20][20];

// ★ 超軽量化した履歴ツリー用ノード（盤面を持たない）
struct HistoryNode {
    int parent_idx;
    int op_start;
    int op_end;
};

// ★ 盤面を持つ状態（現在のビーム幅の分だけ生成され、すぐ捨てられる）
struct State {
    int history_idx; // 自分が属する履歴ノードへのポインタ
    int current_target;
    int total_ops;
    long long eval_score;
    int grid[20][20];
    int pos_r[400];
    int pos_c[400];
};

vector<HistoryNode> history_tree;

// 操作履歴は1つの巨大な1次元配列に平坦化して書き込む（vectorのメモリ確保オーバーヘッドを無くす）
int global_ops_m[20000000];
int global_ops_d[20000000];
int global_ops_cnt = 0;

int total_macro_steps = 0;
int total_restarts = 0;

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
            int r = belt_r[m][i], c = belt_c[m][i];
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
        int r = p.first, c = p.second;
        q.pop();
        for (int i = 0; i < cb_cnt[r][c]; ++i) {
            int m = cb_m[r][c][i], idx = cb_idx[r][c][i];
            int p_idx1 = (idx - 1 + 40) % 40;
            int pr1 = belt_r[m][p_idx1], pc1 = belt_c[m][p_idx1];
            if (dist_to_exit[pr1][pc1] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr1][pc1] = dist_to_exit[r][c] + 1;
                q.push({pr1, pc1});
            }
            int p_idx2 = (idx + 1) % 40;
            int pr2 = belt_r[m][p_idx2], pc2 = belt_c[m][p_idx2];
            if (dist_to_exit[pr2][pc2] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr2][pc2] = dist_to_exit[r][c] + 1;
                q.push({pr2, pc2});
            }
        }
    }
}

State run_macro_step(const State& parent_state) {
    State child = parent_state;
    int K = child.current_target;
    int op_start = global_ops_cnt; // 操作の記録開始位置

    while (child.current_target == K) {
        int tr = child.pos_r[K];
        int tc = child.pos_c[K];

        if (tr == -1) { child.current_target++; break; }

        int current_dist = dist_to_exit[tr][tc];
        int best_m = -1, best_d = 0;
        long long best_score = 2e18;

        for (int i = 0; i < cb_cnt[tr][tc]; ++i) {
            int m = cb_m[tr][tc][i];
            int c_idx = cb_idx[tr][tc][i];

            for (int d_idx = 0; d_idx < 2; ++d_idx) {
                int d = (d_idx == 0) ? 1 : -1;
                int n_idx = (c_idx + d + 40) % 40;
                int nr = belt_r[m][n_idx];
                int nc = belt_c[m][n_idx];

                if (dist_to_exit[nr][nc] >= current_dist) continue;

                long long sub_score = 0;
                long long weights[] = {10000LL, 5000LL, 1000LL, 100LL}; 
                for (int offset = 1; offset <= 4; ++offset) {
                    int check_id = K + offset;
                    if (check_id >= 400 || child.pos_r[check_id] == -1) continue;
                    int cr = child.pos_r[check_id], cc = child.pos_c[check_id];

                    bool on_belt = false; int cidx = -1;
                    for (int l = 0; l < cb_cnt[cr][cc]; ++l) {
                        if (cb_m[cr][cc][l] == m) { on_belt = true; cidx = cb_idx[cr][cc][l]; break; }
                    }
                    if (on_belt) {
                        int nx = (cidx + d + 40) % 40;
                        cr = belt_r[m][nx]; cc = belt_c[m][nx];
                    }
                    sub_score += weights[offset - 1] * dist_to_exit[cr][cc];
                }

                sub_score += xor128() % 8000;

                if (sub_score < best_score) {
                    best_score = sub_score;
                    best_m = m; best_d = d;
                }
            }
        }

        if (best_m == -1) { best_m = cb_m[tr][tc][0]; best_d = 1; }

        int temp[40];
        for(int i=0; i<40; ++i) temp[i] = child.grid[belt_r[best_m][i]][belt_c[best_m][i]];
        for(int i=0; i<40; ++i) {
            int new_idx = (i + best_d + 40) % 40;
            int box_id = temp[i];
            child.grid[belt_r[best_m][new_idx]][belt_c[best_m][new_idx]] = box_id;
            if (box_id != -1) {
                child.pos_r[box_id] = belt_r[best_m][new_idx];
                child.pos_c[box_id] = belt_c[best_m][new_idx];
            }
        }

        // 操作をグローバル配列に記録
        global_ops_m[global_ops_cnt] = best_m;
        global_ops_d[global_ops_cnt] = best_d;
        global_ops_cnt++;
        child.total_ops++;

        while (child.current_target < 400 && child.grid[0][10] == child.current_target) {
            child.grid[0][10] = -1;
            child.pos_r[child.current_target] = -1;
            child.pos_c[child.current_target] = -1;
            child.current_target++;
        }
    }

    // ★ シミュレーション完了後、軽量な「履歴ノード」だけを作成してツリーに繋ぐ
    HistoryNode hn;
    hn.parent_idx = child.history_idx;
    hn.op_start = op_start;
    hn.op_end = global_ops_cnt;
    history_tree.push_back(hn);
    
    // 子状態のポインタを更新
    child.history_idx = history_tree.size() - 1;

    // 評価値の計算
    child.eval_score = (long long)child.total_ops * 100LL; 
    long long w[] = {120, 80, 40, 10}; 
    for (int i = 0; i < 4; ++i) {
        int t = child.current_target + i;
        if (t < 400 && child.pos_r[t] != -1) {
            child.eval_score += w[i] * dist_to_exit[child.pos_r[t]][child.pos_c[t]];
        }
    }

    return child;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);
    auto start_time = steady_clock::now();

    int dummy_N;
    if (!(cin >> dummy_N)) return 0;

    for(int i = 0; i < 400; ++i) { init_pos_r[i] = -1; init_pos_c[i] = -1; }
    for (int i = 0; i < 20; ++i) {
        for (int j = 0; j < 20; ++j) {
            int v; cin >> v;
            init_grid[i][j] = v;
            if (v != -1) { init_pos_r[v] = i; init_pos_c[v] = j; }
        }
    }

    init_belts();
    init_bfs();

    vector<pair<int, int>> best_global_ops;
    int best_total_ops = 1e9;
    bool time_up = false;

    while (true) {
        auto current_time = steady_clock::now();
        if (duration_cast<milliseconds>(current_time - start_time).count() > 1850) {
            break;
        }

        total_restarts++;
        history_tree.clear();
        global_ops_cnt = 0;
        
        State root;
        root.history_idx = -1;
        root.total_ops = 0;
        root.current_target = 0;
        for (int i = 0; i < 20; ++i) for (int j = 0; j < 20; ++j) root.grid[i][j] = init_grid[i][j];
        for (int i = 0; i < 400; ++i) { root.pos_r[i] = init_pos_r[i]; root.pos_c[i] = init_pos_c[i]; }
        
        while (root.current_target < 400 && root.grid[0][10] == root.current_target) {
            root.grid[0][10] = -1;
            root.pos_r[root.current_target] = -1;
            root.pos_c[root.current_target] = -1;
            root.current_target++;
        }
        root.eval_score = 0;

        vector<State> current_beam;
        current_beam.push_back(root);

        int BEAM_WIDTH = 25; 
        int BRANCHES = 10;   

        while (true) {
            current_time = steady_clock::now();
            if (duration_cast<milliseconds>(current_time - start_time).count() > 1850) {
                time_up = true;
                break;
            }

            int min_target = 400;
            for (const auto& st : current_beam) min_target = min(min_target, st.current_target);
            if (min_target >= 400) break;

            vector<State> next_candidates;
            for (const auto& st : current_beam) {
                if (st.current_target > min_target) {
                    next_candidates.push_back(st);
                    continue;
                }
                for (int r = 0; r < BRANCHES; ++r) {
                    State new_state = run_macro_step(st);
                    next_candidates.push_back(new_state);
                    total_macro_steps++;
                }
            }

            sort(next_candidates.begin(), next_candidates.end(), [](const State& a, const State& b) {
                return a.eval_score < b.eval_score;
            });

            int W = min((int)next_candidates.size(), BEAM_WIDTH);
            current_beam.assign(next_candidates.begin(), next_candidates.begin() + W);
        }

        // 最良状態の検索
        int best_idx = 0;
        for (int i = 1; i < current_beam.size(); ++i) {
            if (current_beam[i].current_target > current_beam[best_idx].current_target || 
               (current_beam[i].current_target == current_beam[best_idx].current_target && current_beam[i].total_ops < current_beam[best_idx].total_ops)) {
                best_idx = i;
            }
        }

        // ★ 履歴ツリーを辿って操作手順を復元する
        vector<pair<int, int>> final_ops;
        vector<int> path;
        int curr = current_beam[best_idx].history_idx;
        while (curr != -1) {
            path.push_back(curr);
            curr = history_tree[curr].parent_idx;
        }
        reverse(path.begin(), path.end());
        for (int p : path) {
            for (int i = history_tree[p].op_start; i < history_tree[p].op_end; ++i) {
                final_ops.push_back({global_ops_m[i], global_ops_d[i]});
            }
        }

        // 途中で時間切れになった場合は、残りの箱を貪欲に運んで補完
        State final_state = current_beam[best_idx];
        while (final_state.current_target < 400) {
            int K = final_state.current_target;
            if (final_state.pos_r[K] == -1) { final_state.current_target++; continue; }

            int tr = final_state.pos_r[K], tc = final_state.pos_c[K];
            int current_dist = dist_to_exit[tr][tc];
            int best_m = -1, best_d = 0;
            long long best_score = 2e18;

            for (int i = 0; i < cb_cnt[tr][tc]; ++i) {
                int m = cb_m[tr][tc][i];
                int c_idx = cb_idx[tr][tc][i];
                for (int d_idx = 0; d_idx < 2; ++d_idx) {
                    int d = (d_idx == 0) ? 1 : -1;
                    int n_idx = (c_idx + d + 40) % 40;
                    int nr = belt_r[m][n_idx], nc = belt_c[m][n_idx];
                    if (dist_to_exit[nr][nc] >= current_dist) continue;

                    long long sub_score = 0;
                    for (int offset = 1; offset <= 2; ++offset) {
                        int check_id = K + offset;
                        if (check_id >= 400 || final_state.pos_r[check_id] == -1) continue;
                        int cr = final_state.pos_r[check_id], cc = final_state.pos_c[check_id];
                        bool on_belt = false; int cidx = -1;
                        for (int l = 0; l < cb_cnt[cr][cc]; ++l) {
                            if (cb_m[cr][cc][l] == m) { on_belt = true; cidx = cb_idx[cr][cc][l]; break; }
                        }
                        if (on_belt) {
                            int nx = (cidx + d + 40) % 40;
                            cr = belt_r[m][nx]; cc = belt_c[m][nx];
                        }
                        sub_score += dist_to_exit[cr][cc];
                    }
                    if (sub_score < best_score) { best_score = sub_score; best_m = m; best_d = d; }
                }
            }
            if (best_m == -1) { best_m = cb_m[tr][tc][0]; best_d = 1; }

            int temp[40];
            for(int i=0; i<40; ++i) temp[i] = final_state.grid[belt_r[best_m][i]][belt_c[best_m][i]];
            for(int i=0; i<40; ++i) {
                int new_idx = (i + best_d + 40) % 40;
                int box_id = temp[i];
                final_state.grid[belt_r[best_m][new_idx]][belt_c[best_m][new_idx]] = box_id;
                if (box_id != -1) {
                    final_state.pos_r[box_id] = belt_r[best_m][new_idx];
                    final_state.pos_c[box_id] = belt_c[best_m][new_idx];
                }
            }
            final_ops.push_back({best_m, best_d});
            while (final_state.current_target < 400 && final_state.grid[0][10] == final_state.current_target) {
                final_state.grid[0][10] = -1;
                final_state.pos_r[final_state.current_target] = -1;
                final_state.pos_c[final_state.current_target] = -1;
                final_state.current_target++;
            }
        }

        if (final_ops.size() < best_total_ops) {
            best_total_ops = final_ops.size();
            best_global_ops = final_ops;
        }

        if (time_up) break;
    }

    // --- 出力フェーズ ---
    cout << 20 << "\n";
    for (int m = 0; m < 20; ++m) {
        cout << 40;
        for (int i = 0; i < 40; ++i) {
            cout << " " << belt_r[m][i] << " " << belt_c[m][i];
        }
        cout << "\n";
    }
    
    cout << best_global_ops.size() << "\n";
    for (auto op : best_global_ops) {
        cout << op.first << " " << op.second << "\n";
    }

    cerr << "Time Limit Hit: " << (time_up ? "Yes" : "No") << "\n";
    cerr << "Total Restarts (Beam Search runs): " << total_restarts << "\n";
    cerr << "Total Macro Simulations: " << total_macro_steps << "\n";
    cerr << "Best Operations Count: " << best_total_ops << "\n";

    return 0;
}