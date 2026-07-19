#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <algorithm>
#include <chrono>
#include <random>
#include <cstring> // memcpy用

using namespace std;

const int N = 20;
const double TIME_LIMIT_GREEDY = 0.8; // 貪欲多スタートフェーズの時間（秒）
const double TIME_LIMIT_TOTAL = 1.65;  // 全体の時間制限（秒）

// 実験で最も成果を出した初期の黄金比率を固定適用
const int COEFF_DIST = 1;       
const int COEFF_MATCHED = 50;  
const int COEFF_DESTROY = 200;  

struct Operation {
    char dir; 
    int r, c, h, w;
};

int a[N][N];
int init_a[N][N]; // 貪欲の多スタート用に初期盤面を保存する配列
string V[N];
string H[N - 1];

// 各カードの目的地へのマンハッタン距離の総和を計算
int calc_total_distance() {
    int dist = 0;
    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) {
            int target_r = a[i][j] / N;
            int target_c = a[i][j] % N;
            dist += abs(i - target_r) + abs(j - target_c);
        }
    }
    return dist;
}

// 完全に位置が一致しているマスの数を計算
int calc_matched_cells() {
    int matched = 0;
    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) {
            if (a[i][j] == i * N + j) {
                matched++;
            }
        }
    }
    return matched;
}

// 操作の適用
void apply_op(const Operation& op) {
    if (op.dir == 'V') {
        int half_h = op.h / 2;
        for (int x = 0; x < half_h; ++x) {
            for (int y = 0; y < op.w; ++y) {
                swap(a[op.r + x][op.c + y], a[op.r + half_h + x][op.c + y]);
            }
        }
    } else {
        int half_w = op.w / 2;
        for (int x = 0; x < op.h; ++x) {
            for (int y = 0; y < half_w; ++y) {
                swap(a[op.r + x][op.c + y], a[op.r + x][op.c + half_w + y]);
            }
        }
    }
}

// 長方形の内部に壁が存在するかどうかを判定
bool has_wall(int r, int c, int h, int w) {
    for (int i = r; i < r + h - 1; ++i) {
        for (int j = c; j < c + w; ++j) {
            if (H[i][j] == '1') return true;
        }
    }
    for (int i = r; i < r + h; ++i) {
        for (int j = c; j < c + w - 1; ++j) {
            if (V[i][j] == '1') return true;
        }
    }
    return false;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    auto start_time = chrono::high_resolution_clock::now();

    int n_in;
    cin >> n_in;

    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) {
            cin >> a[i][j];
            init_a[i][j] = a[i][j]; // 初期状態をコピー退避
        }
    }
    for (int i = 0; i < N; ++i) cin >> V[i];
    for (int i = 0; i < N - 1; ++i) cin >> H[i];

    // 有効操作の列挙
    vector<Operation> valid_ops;
    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int h = 1; r + h <= N; ++h) {
                for (int w = 1; c + w <= N; ++w) {
                    if (has_wall(r, c, h, w)) continue;
                    if (h % 2 == 0) valid_ops.push_back({'V', r, c, h, w});
                    if (w % 2 == 0) valid_ops.push_back({'H', r, c, h, w});
                }
            }
        }
    }

    // 各マスをカバーする有効操作のインデックスマップ
    vector<int> ops_covering[N][N];
    for (int i = 0; i < (int)valid_ops.size(); ++i) {
        const auto& op = valid_ops[i];
        for (int r = op.r; r < op.r + op.h; ++r) {
            for (int c = op.c; c < op.c + op.w; ++c) {
                ops_covering[r][c].push_back(i);
            }
        }
    }

    mt19937 rng(42);
    
    vector<Operation> best_greedy_ans;
    int best_greedy_dist = 999999;
    int best_greedy_matched = -1;

    // ---- 【Phase 1】多スタート乱択Greedy (0.0秒 〜 0.8秒) ----
    cerr << "[Phase 1] Starting Multi-Start Random Tie-Break Greedy..." << endl;
    int greedy_runs = 0;

    while (true) {
        auto now = chrono::high_resolution_clock::now();
        double elapsed = chrono::duration_cast<chrono::milliseconds>(now - start_time).count() / 1000.0;
        if (elapsed > TIME_LIMIT_GREEDY) break;

        greedy_runs++;
        memcpy(a, init_a, sizeof(a)); 
        
        vector<Operation> current_ops;
        int current_dist_greedy = calc_total_distance();

        while (current_ops.size() < 100000) {
            Operation best_op;
            long long best_score = -999999;
            vector<Operation> candidates;
            vector<int> cand_dist_diff;

            for (const auto& op : valid_ops) {
                int before_d = 0; int before_m = 0;
                for (int r = op.r; r < op.r + op.h; ++r) {
                    for (int c = op.c; c < op.c + op.w; ++c) {
                        if (a[r][c] == r * N + c) before_m++;
                        int tr = a[r][c] / N; int tc = a[r][c] % N;
                        before_d += abs(r - tr) + abs(c - tc);
                    }
                }

                apply_op(op); 

                int after_d = 0; int after_m = 0;
                for (int r = op.r; r < op.r + op.h; ++r) {
                    for (int c = op.c; c < op.c + op.w; ++c) {
                        if (a[r][c] == r * N + c) after_m++;
                        int tr = a[r][c] / N; int tc = a[r][c] % N;
                        after_d += abs(r - tr) + abs(c - tc);
                    }
                }

                apply_op(op); 

                int dist_diff = before_d - after_d; 
                int matched_diff = after_m - before_m; 

                long long op_score = (long long)dist_diff * COEFF_DIST;
                if (matched_diff > 0) op_score += (long long)matched_diff * COEFF_MATCHED;
                if (matched_diff < 0) op_score += (long long)matched_diff * COEFF_DESTROY;

                if (op_score > best_score) {
                    best_score = op_score;
                    candidates.clear();
                    cand_dist_diff.clear();
                    candidates.push_back(op);
                    cand_dist_diff.push_back(dist_diff);
                } else if (op_score == best_score) {
                    candidates.push_back(op);
                    cand_dist_diff.push_back(dist_diff);
                }
            }

            if (best_score <= 0 || candidates.empty()) {
                break; 
            }

            int idx = rng() % candidates.size();
            best_op = candidates[idx];
            int best_dist_diff = cand_dist_diff[idx];

            apply_op(best_op);
            current_dist_greedy -= best_dist_diff;
            current_ops.push_back(best_op);
        }

        int current_matched_greedy = calc_matched_cells();
        
        if (current_matched_greedy > best_greedy_matched || 
           (current_matched_greedy == best_greedy_matched && current_dist_greedy < best_greedy_dist)) {
            best_greedy_matched = current_matched_greedy;
            best_greedy_dist = current_dist_greedy;
            best_greedy_ans = current_ops;
        }
    }

    cerr << "[Phase 1] Finished. Total Runs: " << greedy_runs 
         << " | Best Greedy Matched: " << best_greedy_matched 
         << " | Best Greedy Dist: " << best_greedy_dist << endl;


    // ---- 【Phase 2】4近傍確率ブレンド焼きなまし法 (0.8秒 〜 1.85秒) ----
    cerr << "[Phase 2] Starting 4-Neighborhood Blended SA..." << endl;
    
    memcpy(a, init_a, sizeof(a));
    for (const auto& op : best_greedy_ans) {
        apply_op(op);
    }

    vector<Operation> ans = best_greedy_ans;
    int current_dist = best_greedy_dist;
    int current_matched = best_greedy_matched;

    vector<Operation> best_ans = ans;
    int best_dist = current_dist;
    int best_matched = current_matched;

    long long current_score = (long long)current_dist + (long long)(N * N - current_matched) * 1000;
    long long best_score = current_score;

    double start_temp = 5.0; 
    double end_temp = 0.1;
    long long loop_count = 0;
    double elapsed = 0.0;

    while (true) {
        loop_count++;
        
        if ((loop_count & 1023) == 0) {
            auto now = chrono::high_resolution_clock::now();
            elapsed = chrono::duration_cast<chrono::milliseconds>(now - start_time).count() / 1000.0;
            if (elapsed > TIME_LIMIT_TOTAL) break;
        }
        
        double progress = (elapsed - TIME_LIMIT_GREEDY) / (TIME_LIMIT_TOTAL - TIME_LIMIT_GREEDY);
        progress = max(0.0, min(1.0, progress));
        double temp = start_temp * pow(end_temp / start_temp, progress);

        // 【改善】4つの近傍を確率でブレンド (0:完全ランダム追加(65%), 1:末尾削除(20%), 2:未一致起点追加(10%), 3:途中置換(5%))
        int type = 0;
        int rand_val = rng() % 100;
        if (ans.empty()) {
            type = (rand_val < 85) ? 0 : 2; // 空の時は追加近傍のみ
        } else {
            if (rand_val < 65) type = 0;
            else if (rand_val < 85) type = 1;
            else if (rand_val < 95) type = 2;
            else type = 3;
        }

        if (type == 0 || type == 2) {
            // 末尾への追加近傍
            if (ans.size() >= 100000) continue;

            Operation op;
            if (type == 2) {
                // 未一致起点サンプリング
                int r = 0, c = 0;
                bool found_unmatched = false;
                for (int retry = 0; retry < 50; ++retry) {
                    int rr = rng() % N; int cc = rng() % N;
                    if (a[rr][cc] != rr * N + cc) {
                        r = rr; c = cc; found_unmatched = true;
                        break;
                    }
                }
                if (found_unmatched && !ops_covering[r][c].empty()) {
                    op = valid_ops[ops_covering[r][c][rng() % ops_covering[r][c].size()]];
                } else {
                    op = valid_ops[rng() % valid_ops.size()];
                }
            } else {
                // 完全ランダムサンプリング
                op = valid_ops[rng() % valid_ops.size()];
            }

            int before_d = 0; int before_m = 0;
            for (int r_op = op.r; r_op < op.r + op.h; ++r_op) {
                for (int c_op = op.c; c_op < op.c + op.w; ++c_op) {
                    if (a[r_op][c_op] == r_op * N + c_op) before_m++;
                    int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                    before_d += abs(r_op - tr) + abs(c_op - tc);
                }
            }

            apply_op(op);

            int after_d = 0; int after_m = 0;
            for (int r_op = op.r; r_op < op.r + op.h; ++r_op) {
                for (int c_op = op.c; c_op < op.c + op.w; ++c_op) {
                    if (a[r_op][c_op] == r_op * N + c_op) after_m++;
                    int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                    after_d += abs(r_op - tr) + abs(c_op - tc);
                }
            }

            int next_dist = current_dist - (before_d - after_d);
            int next_matched = current_matched - (before_m - after_m);
            long long next_score = (long long)next_dist + (long long)(N * N - next_matched) * 1000;
            long long score_diff = current_score - next_score; 

            if (score_diff >= 0 || (double)rng() / mt19937::max() < exp((double)score_diff / temp)) {
                current_dist = next_dist;
                current_matched = next_matched;
                current_score = next_score;
                ans.push_back(op);

                if (current_score < best_score) {
                    best_score = current_score;
                    best_dist = current_dist;
                    best_matched = next_matched;
                    best_ans = ans;
                }
            } else {
                apply_op(op);
            }
        } 
        else if (type == 1) {
            // 最後の1手を戻す（削除）近傍
            Operation op = ans.back();

            int before_d = 0; int before_m = 0;
            for (int r_op = op.r; r_op < op.r + op.h; ++r_op) {
                for (int c_op = op.c; c_op < op.c + op.w; ++c_op) {
                    if (a[r_op][c_op] == r_op * N + c_op) before_m++;
                    int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                    before_d += abs(r_op - tr) + abs(c_op - tc);
                }
            }

            apply_op(op); 

            int after_d = 0; int after_m = 0;
            for (int r_op = op.r; r_op < op.r + op.h; ++r_op) {
                for (int c_op = op.c; c_op < op.c + op.w; ++c_op) {
                    if (a[r_op][c_op] == r_op * N + c_op) after_m++;
                    int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                    after_d += abs(r_op - tr) + abs(c_op - tc);
                }
            }

            int next_dist = current_dist - (before_d - after_d);
            int next_matched = current_matched - (before_m - after_m);
            long long next_score = (long long)next_dist + (long long)(N * N - next_matched) * 1000;
            long long score_diff = current_score - next_score;

            if (score_diff >= 0 || (double)rng() / mt19937::max() < exp((double)score_diff / temp)) {
                current_dist = next_dist;
                current_matched = next_matched;
                current_score = next_score;
                ans.pop_back();

                if (current_score < best_score) {
                    best_score = current_score;
                    best_dist = current_dist;
                    best_matched = next_matched;
                    best_ans = ans;
                }
            } else {
                apply_op(op);
            }
        }
        else if (type == 3) {
            // 【復活】操作列の途中置換近傍（比率5%）
            int idx = rng() % ans.size();
            Operation old_op = ans[idx];

            // 状態の完全退避
            long long orig_score = current_score;
            int orig_dist = current_dist;
            int orig_matched = current_matched;

            // 1. 末尾から idx まで高速差分巻き戻し
            for (int i = (int)ans.size() - 1; i >= idx; --i) {
                int after_d = 0; int after_m = 0;
                for (int r_op = ans[i].r; r_op < ans[i].r + ans[i].h; ++r_op) {
                    for (int c_op = ans[i].c; c_op < ans[i].c + ans[i].w; ++c_op) {
                        if (a[r_op][c_op] == r_op * N + c_op) after_m++;
                        int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                        after_d += abs(r_op - tr) + abs(c_op - tc);
                    }
                }
                apply_op(ans[i]); 
                int before_d = 0; int before_m = 0;
                for (int r_op = ans[i].r; r_op < ans[i].r + ans[i].h; ++r_op) {
                    for (int c_op = ans[i].c; c_op < ans[i].c + ans[i].w; ++c_op) {
                        if (a[r_op][c_op] == r_op * N + c_op) before_m++;
                        int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                        before_d += abs(r_op - tr) + abs(c_op - tc);
                    }
                }
                current_dist += (before_d - after_d);
                current_matched += (before_m - after_m);
            }

            // 2. 差し替える新しい操作を完全ランダム有効操作から抽選
            Operation new_op = valid_ops[rng() % valid_ops.size()];
            ans[idx] = new_op;

            // 3. idx から末尾まで順に高速差分再適用
            for (int i = idx; i < (int)ans.size(); ++i) {
                int before_d = 0; int before_m = 0;
                for (int r_op = ans[i].r; r_op < ans[i].r + ans[i].h; ++r_op) {
                    for (int c_op = ans[i].c; c_op < ans[i].c + ans[i].w; ++c_op) {
                        if (a[r_op][c_op] == r_op * N + c_op) before_m++;
                        int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                        before_d += abs(r_op - tr) + abs(c_op - tc);
                    }
                }
                apply_op(ans[i]); 
                int after_d = 0; int after_m = 0;
                for (int r_op = ans[i].r; r_op < ans[i].r + ans[i].h; ++r_op) {
                    for (int c_op = ans[i].c; c_op < ans[i].c + ans[i].w; ++c_op) {
                        if (a[r_op][c_op] == r_op * N + c_op) after_m++;
                        int tr = a[r_op][c_op] / N; int tc = a[r_op][c_op] % N;
                        after_d += abs(r_op - tr) + abs(c_op - tc);
                    }
                }
                current_dist -= (before_d - after_d);
                current_matched -= (before_m - after_m);
            }

            current_score = (long long)current_dist + (long long)(N * N - current_matched) * 1000;
            long long score_diff = orig_score - current_score;

            if (score_diff >= 0 || (double)rng() / mt19937::max() < exp((double)score_diff / temp)) {
                if (current_score < best_score) {
                    best_score = current_score;
                    best_dist = current_dist;
                    best_matched = current_matched;
                    best_ans = ans;
                }
            } else {
                // 拒否された場合は最小手数のシミュレーションで元の状態へ完全復原
                for (int i = (int)ans.size() - 1; i >= idx; --i) { apply_op(ans[i]); }
                ans[idx] = old_op;
                for (int i = idx; i < (int)ans.size(); ++i) { apply_op(ans[i]); }
                current_score = orig_score;
                current_dist = orig_dist;
                current_matched = orig_matched;
            }
        }

        if (loop_count % 2000000 == 0) {
            char log_buf[256];
            snprintf(log_buf, sizeof(log_buf), 
                     "[SA Debug] Loops: %lld | Cur Dist: %d | Best Dist: %d | Best Ops: %d | Matched: %d | Temp: %.4f",
                     loop_count, current_dist, best_dist, (int)best_ans.size(), best_matched, temp);
            cerr << log_buf << endl;
        }
    }

    cerr << "[Final Debug] Total SA Loops: " << loop_count 
         << " | Final Best Dist: " << best_dist 
         << " | Final Best Ops: " << best_ans.size() 
         << " | Final Best Matched: " << best_matched << endl;

    for (const auto& op : best_ans) {
        cout << op.dir << " " << op.r << " " << op.c << " " << op.h << " " << op.w << "\n";
    }
    cout << flush;

    return 0;
}