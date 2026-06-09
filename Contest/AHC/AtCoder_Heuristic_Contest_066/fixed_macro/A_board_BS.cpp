#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <chrono>
#include <unordered_set>
#include <string_view>
#include <random>

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
int dist_map[20][20][20][20];
int dist_apsp[20][20][4][20][20][4];
string path_apsp[20][20][4][20][20][4]; 

bool can_move(int r, int c, int d) {
    if (d == 0) return (c + 1 < N && v_walls[r][c] == '0');
    if (d == 1) return (r + 1 < N && h_walls[r][c] == '0');
    if (d == 2) return (c - 1 >= 0 && v_walls[r][c - 1] == '0');
    if (d == 3) return (r - 1 >= 0 && h_walls[r - 1][c] == '0');
    return false;
}

// 直進優遇パスの完全事前計算
void precompute_apsp() {
    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int i = 0; i < N; ++i) for (int j = 0; j < N; ++j) dist_map[r][c][i][j] = 1e9;
            dist_map[r][c][r][c] = 0;
            queue<pair<int, int>> q; q.push({r, c});
            while (!q.empty()) {
                auto [cr, cc] = q.front(); q.pop();
                for (int d = 0; d < 4; ++d) {
                    if (can_move(cr, cc, d)) {
                        int nr = cr + dr[d], nc = cc + dc[d];
                        if (dist_map[r][c][nr][nc] > dist_map[r][c][cr][cc] + 1) {
                            dist_map[r][c][nr][nc] = dist_map[r][c][cr][cc] + 1; q.push({nr, nc});
                        }
                    }
                }
            }
        }
    }

    static short prev_r[20][20][4], prev_c[20][20][4], prev_d[20][20][4];
    static char m_char[20][20][4];

    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int d = 0; d < 4; ++d) {
                for(int i=0; i<N; i++) for(int j=0; j<N; j++) for(int k=0; k<4; k++) dist_apsp[r][c][d][i][j][k] = 1e9;
                
                dist_apsp[r][c][d][r][c][d] = 0;
                priority_queue<PQNode, vector<PQNode>, greater<PQNode>> pq; pq.push({0, r, c, d});
                
                while (!pq.empty()) {
                    PQNode u = pq.top(); pq.pop();
                    if (u.dist > dist_apsp[r][c][d][u.r][u.c][u.d]) continue;
                    
                    if (can_move(u.r, u.c, u.d)) {
                        int nr = u.r + dr[u.d], nc = u.c + dc[u.d];
                        if (dist_apsp[r][c][d][nr][nc][u.d] > u.dist + 1000) {
                            dist_apsp[r][c][d][nr][nc][u.d] = u.dist + 1000;
                            prev_r[nr][nc][u.d] = u.r; prev_c[nr][nc][u.d] = u.c; prev_d[nr][nc][u.d] = u.d;
                            m_char[nr][nc][u.d] = 'F'; pq.push({u.dist + 1000, nr, nc, u.d});
                        }
                    }
                    int nd_r = (u.d + 1) % 4;
                    if (dist_apsp[r][c][d][u.r][u.c][nd_r] > u.dist + 1001) {
                        dist_apsp[r][c][d][u.r][u.c][nd_r] = u.dist + 1001;
                        prev_r[u.r][u.c][nd_r] = u.r; prev_c[u.r][u.c][nd_r] = u.c; prev_d[u.r][u.c][nd_r] = u.d;
                        m_char[u.r][u.c][nd_r] = 'R'; pq.push({u.dist + 1001, u.r, u.c, nd_r});
                    }
                    int nd_l = (u.d + 3) % 4;
                    if (dist_apsp[r][c][d][u.r][u.c][nd_l] > u.dist + 1001) {
                        dist_apsp[r][c][d][u.r][u.c][nd_l] = u.dist + 1001;
                        prev_r[u.r][u.c][nd_l] = u.r; prev_c[u.r][u.c][nd_l] = u.c; prev_d[u.r][u.c][nd_l] = u.d;
                        m_char[u.r][u.c][nd_l] = 'L'; pq.push({u.dist + 1001, u.r, u.c, nd_l});
                    }
                }

                for (int i = 0; i < N; ++i) {
                    for (int j = 0; j < N; ++j) {
                        for (int k = 0; k < 4; ++k) {
                            if (dist_apsp[r][c][d][i][j][k] >= 1e8) continue;
                            string path = ""; int cr = i, cc = j, cd = k;
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

// 2. 最強の重み付けバケツリレーシミュレータ
struct SimResult { string history; int score; };

SimResult simulate(const vector<int>& W, int swap_thresh) {
    int b_r[45], b_c[45];
    bool in_basket[45];
    int ball_at[20][20];
    for(int r=0; r<N; r++) for(int c=0; c<N; c++) ball_at[r][c] = -1;
    for(int i=0; i<M; i++) {
        b_r[i] = balls[i].r; b_c[i] = balls[i].c;
        in_basket[i] = false;
        ball_at[b_r[i]][b_c[i]] = i;
    }

    int holding = -1;
    int cr = 0, cc = 0, cd = 0;
    string history = ""; history.reserve(2000);
    int turns = 0, loop_count = 0, unbasketed = M;

    while(unbasketed > 0) {
        if (loop_count++ > 2000) return {string(10000, 'F'), (int)1e9}; // 無限ループ回避

        if (holding == -1) {
            int best_b = -1; int min_score = 1e9;
            for(int i = 0; i < M; i++) {
                if(!in_basket[i]) {
                    // ★ 距離だけでなく、SAで学習した「重み(W)」を加味して目標を決定
                    int score = dist_map[cr][cc][b_r[i]][b_c[i]] * 100 + W[i];
                    if (score < min_score) { min_score = score; best_b = i; }
                }
            }
            
            int tr = b_r[best_b], tc = b_c[best_b];
            if (cr == tr && cc == tc) {
                history += "S"; holding = best_b;
                ball_at[cr][cc] = -1; b_r[best_b] = -1; b_c[best_b] = -1;
                continue;
            }

            int best_nd = 0, min_cost = 1e9;
            for(int i=0; i<4; i++) {
                if (dist_apsp[cr][cc][cd][tr][tc][i] < min_cost) {
                    min_cost = dist_apsp[cr][cc][cd][tr][tc][i]; best_nd = i;
                }
            }
            string p = path_apsp[cr][cc][cd][tr][tc][best_nd];

            bool interrupted = false;
            for(char m : p) {
                history += m;
                if (m == 'R') { cd = (cd+1)%4; turns++; }
                else if (m == 'L') { cd = (cd+3)%4; turns++; }
                else {
                    cr += dr[cd]; cc += dc[cd];
                    int stepped = ball_at[cr][cc];
                    // ★ 空手でボールを踏んだら無条件で拾う（ついで拾い）
                    if (stepped != -1 && !in_basket[stepped]) {
                        history += "S"; holding = stepped;
                        ball_at[cr][cc] = -1; b_r[stepped] = -1; b_c[stepped] = -1;
                        interrupted = true; break;
                    }
                }
            }
            if (!interrupted && cr == tr && cc == tc) {
                history += "S"; holding = best_b;
                ball_at[cr][cc] = -1; b_r[best_b] = -1; b_c[best_b] = -1;
            }
            
        } else {
            int tr = baskets[holding].r, tc = baskets[holding].c;
            if (cr == tr && cc == tc) {
                history += "S"; in_basket[holding] = true; unbasketed--;
                int stepped = ball_at[cr][cc];
                if (stepped != -1 && !in_basket[stepped]) {
                    holding = stepped; ball_at[cr][cc] = -1; b_r[stepped] = -1; b_c[stepped] = -1;
                } else holding = -1;
                continue;
            }

            int best_nd = 0, min_cost = 1e9;
            for(int i=0; i<4; i++) {
                if (dist_apsp[cr][cc][cd][tr][tc][i] < min_cost) {
                    min_cost = dist_apsp[cr][cc][cd][tr][tc][i]; best_nd = i;
                }
            }
            string p = path_apsp[cr][cc][cd][tr][tc][best_nd];

            bool interrupted = false;
            for(char m : p) {
                history += m;
                if (m == 'R') { cd = (cd+1)%4; turns++; }
                else if (m == 'L') { cd = (cd+3)%4; turns++; }
                else {
                    cr += dr[cd]; cc += dc[cd];
                    int stepped = ball_at[cr][cc];
                    if (stepped != -1 && !in_basket[stepped]) {
                        if (cr == tr && cc == tc) { // 目的に着いたらかごに置く
                            history += "S"; in_basket[holding] = true; unbasketed--;
                            holding = stepped; ball_at[cr][cc] = -1; b_r[stepped] = -1; b_c[stepped] = -1;
                            interrupted = true; break;
                        } else {
                            // ★ バケツリレーの判定（学習した重みに基づく）
                            int old_s = dist_map[cr][cc][tr][tc] * 100 + W[holding];
                            int new_s = dist_map[cr][cc][baskets[stepped].r][baskets[stepped].c] * 100 + W[stepped];
                            if (new_s < old_s + swap_thresh) {
                                history += "S";
                                ball_at[cr][cc] = holding; b_r[holding] = cr; b_c[holding] = cc; // 置く
                                holding = stepped; ball_at[cr][cc] = -1; b_r[stepped] = -1; b_c[stepped] = -1; // 拾う
                                interrupted = true; break;
                            }
                        }
                    }
                }
            }
            if (!interrupted && cr == tr && cc == tc) {
                history += "S"; in_basket[holding] = true; unbasketed--;
                int stepped = ball_at[cr][cc];
                if (stepped != -1 && !in_basket[stepped]) {
                    holding = stepped; ball_at[cr][cc] = -1; b_r[stepped] = -1; b_c[stepped] = -1;
                } else holding = -1;
            }
        }
    }
    // 回転（R/L）ペナルティを与えることで、マクロ圧縮しやすい経路を評価上で優遇する
    return {history, (int)history.length() + turns * 3};
}

// 3. マクロ圧縮機能 (高速化版)
string compress_string(const string& s) {
    int n = s.length();
    int best_len = 0, best_pos = 0, max_saved = 0;
    string_view sv(s); unordered_set<string_view> seen;
    
    // 探索上限を30に絞り、高速化
    for (int len = 2; len <= 30; ++len) { 
        seen.clear();
        for (int i = 0; i <= n - len; ++i) {
            string_view sub = sv.substr(i, len);
            if (seen.count(sub)) continue;
            seen.insert(sub);
            int count = 0, j = 0;
            while (j <= n - len) {
                if (sv.substr(j, len) == sub) { count++; j += len; } else j++;
            }
            int saved = (len - 1) * (count - 1) - 2;
            if (saved > max_saved) { max_saved = saved; best_len = len; best_pos = i; }
        }
    }
    if (max_saved <= 0) return s;
    
    string_view best_sub = sv.substr(best_pos, best_len);
    string res = ""; res.reserve(n);
    bool first = true; int i = 0;
    while (i < n) {
        if (i <= n - best_len && sv.substr(i, best_len) == best_sub) {
            if (first) { res += "M"; res += string(best_sub); res += "M"; first = false; } 
            else res += "P";
            i += best_len;
        } else { res += s[i]; i++; }
    }
    return res;
}

// チャンク分割による複数マクロ
string compress_multiple_macros(const string& s) {
    string best_res = s;
    vector<int> chunk_sizes = {250, (int)s.length()}; // 分割を最小限にして時間を節約
    for (int chunk_size : chunk_sizes) {
        string current_res = ""; int n = s.length();
        for (int i = 0; i < n; i += chunk_size) {
            int len = min(chunk_size, n - i);
            current_res += compress_string(s.substr(i, len)); 
        }
        if (current_res.length() < best_res.length()) best_res = current_res;
    }
    return best_res;
}

// 候補保持用
struct Cand {
    string history;
    int score;
    bool operator<(const Cand& o) const { return score < o.score; }
};

int main() {
    auto start_time = chrono::system_clock::now();
    // ★ TLE絶対防止タイマー (SAを1.75秒で打ち切り、残りで圧縮を行う)
    double MAX_SA_TIME = 1.75; 

    ios_base::sync_with_stdio(false); cin.tie(NULL);
    cin >> N >> M >> T;
    v_walls.resize(N); for (int i = 0; i < N; ++i) cin >> v_walls[i];
    h_walls.resize(N - 1); for (int i = 0; i < N - 1; ++i) cin >> h_walls[i];
    balls.resize(M); baskets.resize(M);
    for (int i = 0; i < M; ++i) cin >> balls[i].r >> balls[i].c >> baskets[i].r >> baskets[i].c;

    precompute_apsp();
    
    mt19937 mt(42);
    vector<Cand> top_cands;
    auto add_cand = [&](const string& h, int score) {
        for (auto& c : top_cands) if (c.history == h) return;
        top_cands.push_back({h, score});
        sort(top_cands.begin(), top_cands.end());
        if (top_cands.size() > 2) top_cands.pop_back(); // ★ 時間節約のためトップ2のみ保持
    };

    int total_iter = 0;
    vector<int> W(M, 0);
    int swap_thresh = 0;
    
    SimResult cur_res = simulate(W, swap_thresh);
    int current_score = cur_res.score;
    add_cand(cur_res.history, cur_res.score);
    
    double start_temp = 500.0, end_temp = 0.1;

    // --- メインループ：重み付け SA ---
    while (true) {
        if ((total_iter & 255) == 0) {
            double elapsed = chrono::duration_cast<chrono::duration<double>>(chrono::system_clock::now() - start_time).count();
            if (elapsed > MAX_SA_TIME) break;
        }
        total_iter++;

        vector<int> next_W = W;
        int next_thresh = swap_thresh;
        int type = mt() % 4;

        if (type == 0) {
            next_W[mt() % M] += (mt() % 601) - 300; // 重みを微調整
        } else if (type == 1) {
            next_thresh += (mt() % 401) - 200; // バケツリレーの積極性を微調整
        } else if (type == 2) {
            swap(next_W[mt() % M], next_W[mt() % M]); // 重みをスワップ
        } else {
            for(int k=0; k<3; ++k) next_W[mt() % M] += (mt() % 201) - 100;
        }

        SimResult next_res = simulate(next_W, next_thresh);
        if (next_res.score < 9000) add_cand(next_res.history, next_res.score);
        
        double elapsed = chrono::duration_cast<chrono::duration<double>>(chrono::system_clock::now() - start_time).count();
        double phase_progress = min(1.0, elapsed / MAX_SA_TIME);
        double temp = start_temp + (end_temp - start_temp) * phase_progress;

        if (next_res.score <= current_score || exp((current_score - next_res.score) / temp) > (double)(mt() % 10000) / 10000.0) {
            W = next_W; swap_thresh = next_thresh; current_score = next_res.score;
        }
    }

    // ★ Top-2 遅延マクロ評価: 1.75秒経過後に、選び抜かれた2つの精鋭にのみマクロを適用
    string best_final_ans = "";
    int best_final_len = 1e9;
    for (auto& cand : top_cands) {
        string comp = compress_multiple_macros(cand.history);
        if (comp.length() < best_final_len) {
            best_final_len = comp.length();
            best_final_ans = comp;
        }
    }

    cerr << "[DEBUG] Total Iterations (SA): " << total_iter << "\n";
    cerr << "[DEBUG] Base Score (Length + Penalty): " << top_cands[0].score << "\n";
    cerr << "[DEBUG] Final Score (Length): " << best_final_ans.length() << "\n";

    int output_count = 0;
    for (char c : best_final_ans) {
        if (output_count >= T) break;
        cout << c << "\n";
        output_count++;
    }

    return 0;
}