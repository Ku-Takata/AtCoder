#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <chrono>
#include <random>
#include <cmath>
#include <unordered_set>
#include <string_view>

using namespace std;

const int dr[] = {0, 1, 0, -1};
const int dc[] = {1, 0, -1, 0};

int N, M, T;
vector<string> v_walls, h_walls;

struct Point { int r, c; };
struct PQNode {
    int dist, r, c, d;
    bool operator>(const PQNode& o) const { return dist > o.dist; }
};

vector<Point> balls, baskets;
int dist_apsp[20][20][4][20][20][4];
string path_apsp[20][20][4][20][20][4]; 

bool can_move(int r, int c, int d) {
    if (d == 0) return (c + 1 < N && v_walls[r][c] == '0');
    if (d == 1) return (r + 1 < N && h_walls[r][c] == '0');
    if (d == 2) return (c - 1 >= 0 && v_walls[r][c - 1] == '0');
    if (d == 3) return (r - 1 >= 0 && h_walls[r - 1][c] == '0');
    return false;
}

// コストとパス文字列の事前計算
void precompute_apsp() {
    static short prev_r[20][20][4], prev_c[20][20][4], prev_d[20][20][4];
    static char m_char[20][20][4];

    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int d = 0; d < 4; ++d) {
                for (int i = 0; i < N; ++i)
                    for (int j = 0; j < N; ++j)
                        for (int k = 0; k < 4; ++k)
                            dist_apsp[r][c][d][i][j][k] = 1e9;
                
                dist_apsp[r][c][d][r][c][d] = 0;
                priority_queue<PQNode, vector<PQNode>, greater<PQNode>> pq;
                pq.push({0, r, c, d});
                
                while (!pq.empty()) {
                    PQNode u = pq.top();
                    pq.pop();
                    if (u.dist > dist_apsp[r][c][d][u.r][u.c][u.d]) continue;
                    
                    if (can_move(u.r, u.c, u.d)) {
                        int nr = u.r + dr[u.d], nc = u.c + dc[u.d];
                        if (dist_apsp[r][c][d][nr][nc][u.d] > u.dist + 1000) {
                            dist_apsp[r][c][d][nr][nc][u.d] = u.dist + 1000;
                            prev_r[nr][nc][u.d] = u.r; prev_c[nr][nc][u.d] = u.c; prev_d[nr][nc][u.d] = u.d;
                            m_char[nr][nc][u.d] = 'F';
                            pq.push({u.dist + 1000, nr, nc, u.d});
                        }
                    }
                    int nd_r = (u.d + 1) % 4;
                    if (dist_apsp[r][c][d][u.r][u.c][nd_r] > u.dist + 1001) {
                        dist_apsp[r][c][d][u.r][u.c][nd_r] = u.dist + 1001;
                        prev_r[u.r][u.c][nd_r] = u.r; prev_c[u.r][u.c][nd_r] = u.c; prev_d[u.r][u.c][nd_r] = u.d;
                        m_char[u.r][u.c][nd_r] = 'R';
                        pq.push({u.dist + 1001, u.r, u.c, nd_r});
                    }
                    int nd_l = (u.d + 3) % 4;
                    if (dist_apsp[r][c][d][u.r][u.c][nd_l] > u.dist + 1001) {
                        dist_apsp[r][c][d][u.r][u.c][nd_l] = u.dist + 1001;
                        prev_r[u.r][u.c][nd_l] = u.r; prev_c[u.r][u.c][nd_l] = u.c; prev_d[u.r][u.c][nd_l] = u.d;
                        m_char[u.r][u.c][nd_l] = 'L';
                        pq.push({u.dist + 1001, u.r, u.c, nd_l});
                    }
                }

                for (int i = 0; i < N; ++i) {
                    for (int j = 0; j < N; ++j) {
                        for (int k = 0; k < 4; ++k) {
                            if (dist_apsp[r][c][d][i][j][k] >= 1e8) continue;
                            string path = "";
                            int cr = i, cc = j, cd = k;
                            while (cr != r || cc != c || cd != d) {
                                path += m_char[cr][cc][cd];
                                int pr = prev_r[cr][cc][cd], pc = prev_c[cr][cc][cd], pd = prev_d[cr][cc][cd];
                                cr = pr; cc = pc; cd = pd;
                            }
                            reverse(path.begin(), path.end());
                            path_apsp[r][c][d][i][j][k] = path;
                        }
                    }
                }
            }
        }
    }
}

string compress_string(const string& s) {
    int n = s.length();
    int best_len = 0, best_pos = 0, max_saved = 0;
    string_view sv(s);
    unordered_set<string_view> seen;

    for (int len = 2; len <= 35; ++len) {
        seen.clear();
        for (int i = 0; i <= n - len; ++i) {
            string_view sub = sv.substr(i, len);
            if (seen.count(sub)) continue;
            seen.insert(sub);
            
            int count = 0, j = 0;
            while (j <= n - len) {
                if (sv.substr(j, len) == sub) {
                    count++; j += len;
                } else j++;
            }
            
            int saved = (len - 1) * (count - 1) - 2;
            if (saved > max_saved) {
                max_saved = saved;
                best_len = len; best_pos = i;
            }
        }
    }

    if (max_saved <= 0) return s;

    string_view best_sub = sv.substr(best_pos, best_len);
    string res = "";
    res.reserve(n);
    bool first = true;
    int i = 0;
    while (i < n) {
        if (i <= n - best_len && sv.substr(i, best_len) == best_sub) {
            if (first) {
                res += "M"; res += string(best_sub); res += "M";
                first = false;
            } else res += "P";
            i += best_len;
        } else {
            res += s[i];
            i++;
        }
    }
    return res;
}

// 1. 軽量な評価関数（ベースとなるコストだけを高速に計算）
int evaluate_light(const vector<int>& P) {
    int dp[4] = {0, (int)1e9, (int)1e9, (int)1e9}; 
    int curr_r = 0, curr_c = 0;
    
    for (int i = 0; i < M; ++i) {
        int b = P[i];
        int next_dp[4] = {(int)1e9, (int)1e9, (int)1e9, (int)1e9};
        for (int pd = 0; pd < 4; ++pd) {
            if (dp[pd] > 1e8) continue;
            for (int nd = 0; nd < 4; ++nd) {
                int cost = dist_apsp[curr_r][curr_c][pd][balls[b].r][balls[b].c][nd];
                next_dp[nd] = min(next_dp[nd], dp[pd] + cost + 1000); 
            }
        }
        for (int d = 0; d < 4; ++d) dp[d] = next_dp[d];
        curr_r = balls[b].r; curr_c = balls[b].c;
        
        int next_dp2[4] = {(int)1e9, (int)1e9, (int)1e9, (int)1e9};
        for (int pd = 0; pd < 4; ++pd) {
            if (dp[pd] > 1e8) continue;
            for (int nd = 0; nd < 4; ++nd) {
                int cost = dist_apsp[curr_r][curr_c][pd][baskets[b].r][baskets[b].c][nd];
                next_dp2[nd] = min(next_dp2[nd], dp[pd] + cost + 1000); 
            }
        }
        for (int d = 0; d < 4; ++d) dp[d] = next_dp2[d];
        curr_r = baskets[b].r; curr_c = baskets[b].c;
    }
    
    int ans = 1e9;
    for (int d = 0; d < 4; ++d) ans = min(ans, dp[d]);
    return ans;
}

// 2. 重い評価関数（マクロ圧縮を実際に行い、最終出力の文字数を真のスコアとして返す）
int evaluate_heavy(const vector<int>& P, string& out_ans) {
    int dp[85][4];
    int prev_dir[85][4];
    for(int i = 0; i < 85; i++) for(int d = 0; d < 4; d++) dp[i][d] = 1e9;
    dp[0][0] = 0; 
    
    vector<Point> targets(M * 2 + 1);
    targets[0] = {0, 0};
    for(int i = 0; i < M; i++){
        targets[i*2 + 1] = balls[P[i]];
        targets[i*2 + 2] = baskets[P[i]];
    }
    
    for(int step = 0; step < M * 2; ++step) {
        int cr = targets[step].r, cc = targets[step].c;
        int nr = targets[step+1].r, nc = targets[step+1].c;
        
        for(int pd = 0; pd < 4; ++pd) {
            if (dp[step][pd] >= 1e8) continue;
            for(int nd = 0; nd < 4; ++nd) {
                int cost = dist_apsp[cr][cc][pd][nr][nc][nd];
                if(dp[step+1][nd] > dp[step][pd] + cost + 1000) {
                    dp[step+1][nd] = dp[step][pd] + cost + 1000;
                    prev_dir[step+1][nd] = pd;
                }
            }
        }
    }
    
    int best_d = 0, min_cost = 1e9;
    for(int d = 0; d < 4; d++) {
        if(dp[M*2][d] < min_cost) { min_cost = dp[M*2][d]; best_d = d; }
    }
    
    vector<int> dirs(M * 2 + 1);
    int curr_d = best_d;
    for(int step = M * 2; step >= 0; --step) {
        dirs[step] = curr_d;
        if(step > 0) curr_d = prev_dir[step][curr_d];
    }
    
    string full_path = "";
    for(int step = 0; step < M * 2; ++step) {
        int cr = targets[step].r, cc = targets[step].c, cd = dirs[step];
        int nr = targets[step+1].r, nc = targets[step+1].c, nd = dirs[step+1];
        full_path += path_apsp[cr][cc][cd][nr][nc][nd];
        full_path += "S";
    }
    
    out_ans = compress_string(full_path);
    return out_ans.length(); // 問題に忠実な「圧縮後の文字数」を返す
}

int main() {
    auto start_time = chrono::system_clock::now();
    double MAX_TIME = 1.85;

    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    cin >> N >> M >> T;
    v_walls.resize(N);
    for (int i = 0; i < N; ++i) cin >> v_walls[i];
    h_walls.resize(N - 1);
    for (int i = 0; i < N - 1; ++i) cin >> h_walls[i];

    balls.resize(M); baskets.resize(M);
    for (int i = 0; i < M; ++i) cin >> balls[i].r >> balls[i].c >> baskets[i].r >> baskets[i].c;

    precompute_apsp();
    auto apsp_end_time = chrono::system_clock::now();
    double elapsed_apsp = chrono::duration_cast<chrono::duration<double>>(apsp_end_time - start_time).count();

    mt19937 mt(42);

    // --- 貪欲法による初期解 ---
    vector<int> greedy_P;
    vector<bool> used(M, false);
    int cr = 0, cc = 0;
    int init_dp[4] = {0, (int)1e9, (int)1e9, (int)1e9};

    for (int i = 0; i < M; ++i) {
        int best_b = -1, min_score = 2e9;
        int best_next_dp[4], best_r = 0, best_c = 0;

        for (int b = 0; b < M; ++b) {
            if (used[b]) continue;
            int next_dp[4] = {(int)1e9, (int)1e9, (int)1e9, (int)1e9};
            for (int pd = 0; pd < 4; ++pd) {
                if (init_dp[pd] > 1e8) continue;
                for (int nd = 0; nd < 4; ++nd) {
                    int cost = dist_apsp[cr][cc][pd][balls[b].r][balls[b].c][nd];
                    next_dp[nd] = min(next_dp[nd], init_dp[pd] + cost + 1000);
                }
            }
            int temp_r = balls[b].r, temp_c = balls[b].c;
            int next_dp2[4] = {(int)1e9, (int)1e9, (int)1e9, (int)1e9};
            for (int pd = 0; pd < 4; ++pd) {
                if (next_dp[pd] > 1e8) continue;
                for (int nd = 0; nd < 4; ++nd) {
                    int cost = dist_apsp[temp_r][temp_c][pd][baskets[b].r][baskets[b].c][nd];
                    next_dp2[nd] = min(next_dp2[nd], next_dp[pd] + cost + 1000);
                }
            }
            int score = 1e9;
            for (int d = 0; d < 4; ++d) score = min(score, next_dp2[d]);

            if (score < min_score) {
                min_score = score; best_b = b;
                for (int d = 0; d < 4; ++d) best_next_dp[d] = next_dp2[d];
                best_r = baskets[b].r; best_c = baskets[b].c;
            }
        }
        greedy_P.push_back(best_b);
        used[best_b] = true;
        for (int d = 0; d < 4; ++d) init_dp[d] = best_next_dp[d];
        cr = best_r; cc = best_c;
    }

    // --- 多点スタート焼きなまし ---
    int NUM_RESTARTS = 6;
    double time_per_restart = (MAX_TIME - elapsed_apsp) / NUM_RESTARTS;
    
    vector<int> global_best_P = greedy_P;
    string global_best_str = "";
    int global_best_true_score = evaluate_heavy(greedy_P, global_best_str);
    int total_iter = 0;

    for (int restart = 0; restart < NUM_RESTARTS; ++restart) {
        vector<int> P = (restart == 0) ? greedy_P : global_best_P;
        if (restart > 0) {
            if (restart % 2 == 1) shuffle(P.begin(), P.end(), mt);
            else for(int k = 0; k < M / 4; ++k) swap(P[mt() % M], P[mt() % M]);
        }

        int current_cost = evaluate_light(P);
        string current_best_str = "";
        int current_true_score = evaluate_heavy(P, current_best_str);
        
        vector<int> best_P = P;
        int best_true_score = current_true_score;
        string best_str_in_restart = current_best_str;

        // 温度スケールは「文字数（1〜1000程度）」に合わせて調整
        double start_temp = 10.0;
        double end_temp = 0.01;
        double restart_start_time = elapsed_apsp + restart * time_per_restart;

        while (true) {
            if ((total_iter & 127) == 0) {
                auto current_time = chrono::system_clock::now();
                double elapsed = chrono::duration_cast<chrono::duration<double>>(current_time - start_time).count();
                if (elapsed > restart_start_time + time_per_restart || elapsed > MAX_TIME) break;
            }
            total_iter++;

            int type = mt() % 4; // 0〜3の4種類
            int i = mt() % M;
            int j;
            
            // 賢い近傍の適用
            if (mt() % 2 == 0) j = mt() % M; 
            else { int offset = (mt() % 5) + 1; j = (mt() % 2 == 0) ? (i + offset) % M : (i - offset + M) % M; }
            if (i == j) continue;

            vector<int> next_P = P;
            if (type == 0) {
                swap(next_P[i], next_P[j]);
            } else if (type == 1) {
                if (i > j) swap(i, j);
                reverse(next_P.begin() + i, next_P.begin() + j + 1);
            } else if (type == 2) {
                int val = next_P[i];
                next_P.erase(next_P.begin() + i);
                next_P.insert(next_P.begin() + j, val);
            } else {
                // ★ 新戦略：Block Insert (複数個の塊を別の場所に移動)
                int len = (mt() % 3) + 2; 
                if (i + len < M) {
                    vector<int> block(next_P.begin() + i, next_P.begin() + i + len);
                    next_P.erase(next_P.begin() + i, next_P.begin() + i + len);
                    int ins = mt() % (next_P.size() + 1);
                    next_P.insert(next_P.begin() + ins, block.begin(), block.end());
                } else continue;
            }

            // ★ 遅延評価：まず軽い評価で絶望的なルートを即座に弾く
            int next_cost = evaluate_light(next_P);
            // （2手以上悪化＝2000以上悪化しているルートはマクロを試す価値がないとしてスキップ）
            if (next_cost > current_cost + 2000) continue; 

            // 有望なルートにだけ、重い「マクロ込みの真の評価」を下す
            string next_str = "";
            int next_true_score = evaluate_heavy(next_P, next_str);
            
            auto current_time = chrono::system_clock::now();
            double elapsed = chrono::duration_cast<chrono::duration<double>>(current_time - start_time).count();
            double phase_progress = min(1.0, (elapsed - restart_start_time) / time_per_restart);
            double temp = start_temp + (end_temp - start_temp) * phase_progress;

            // 採用の合否は、常に「マクロ圧縮後の真の文字数」で判定される
            if (next_true_score <= current_true_score || exp((current_true_score - next_true_score) / temp) > (double)(mt() % 10000) / 10000.0) {
                P = next_P;
                current_cost = next_cost;
                current_true_score = next_true_score;
                if (current_true_score < best_true_score) {
                    best_true_score = current_true_score;
                    best_P = P;
                    best_str_in_restart = next_str;
                }
            }
        }

        if (best_true_score < global_best_true_score) {
            global_best_true_score = best_true_score;
            global_best_P = best_P;
            global_best_str = best_str_in_restart;
        }
    }

    // デバッグ出力：純粋な試行回数と、最終的な出力文字数のみを表示
    cerr << "[DEBUG] Total Iterations: " << total_iter << "\n";
    cerr << "[DEBUG] Final Score (Length): " << global_best_true_score << "\n";

    int output_count = 0;
    for (char c : global_best_str) {
        if (output_count >= T) break;
        cout << c << "\n";
        output_count++;
    }

    return 0;
}