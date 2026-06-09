#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <iomanip>
#include <random>

using namespace std;

// -----------------------------------------------------------
// 高速な乱数ジェネレータ
// -----------------------------------------------------------
uint32_t xr_x = 123456789;
uint32_t xr_y = 362436069;
uint32_t xr_z = 521288629;
uint32_t xr_w = 88675123;

void init_rand() {
    std::random_device rd;
    uint32_t s = rd();
    s ^= (uint32_t)std::chrono::system_clock::now().time_since_epoch().count();
    if (s == 0) s = 1;
    xr_x = s;
    xr_y = s * 1812433253U + 1;
    xr_z = xr_y * 1812433253U + 1;
    xr_w = xr_z * 1812433253U + 1;
}

inline uint32_t xor128() {
    uint32_t t = xr_x ^ (xr_x << 11);
    xr_x = xr_y; xr_y = xr_z; xr_z = xr_w;
    return xr_w = (xr_w ^ (xr_w >> 19)) ^ (t ^ (t >> 8));
}

// -----------------------------------------------------------
// チューニングパラメータ
// -----------------------------------------------------------
struct Config {
    static constexpr int MAX_N = 20;
    static constexpr int MAX_M = 40;
    static constexpr int MAX_STATES = 1600;
    static constexpr int INF = 1e9;
    
    static constexpr int TSP_MIN_TIME_MS = 350; 
    static constexpr int TIME_MARGIN_MS  = 20;  

    static constexpr double MACRO_SA_T0 = 40.0;
    static constexpr double MACRO_SA_T1 = 0.1;
    static constexpr double TSP_SA_T0   = 1.0;
    static constexpr double TSP_SA_T1   = 0.001;

    // Eコスト(T制限) ペナルティ 【通常時】 (Tに余裕あり。Aコスト最適化を優先し、ペナルティは通常レベル)
    static constexpr double SAFE_T_RATIO_MACRO_NORMAL = 0.97; 
    static constexpr double PENALTY_MACRO_WARN_NORMAL = 1.0;  
    static constexpr double PENALTY_MACRO_FATAL_NORMAL= 50.0;
    static constexpr double SAFE_T_RATIO_TSP_NORMAL   = 0.95; 
    static constexpr double PENALTY_TSP_WARN_NORMAL   = 1.5;  
    static constexpr double PENALTY_TSP_FATAL_NORMAL  = 10.0; 

    // Eコスト(T制限) ペナルティ 【T下振れ時】 (Tが厳しい！絶対にTを超えないよう早期から警告しペナルティを激増)
    static constexpr double SAFE_T_RATIO_MACRO_LOW_T = 0.70; 
    static constexpr double PENALTY_MACRO_WARN_LOW_T = 3.0;  
    static constexpr double PENALTY_MACRO_FATAL_LOW_T= 500.0;
    static constexpr double SAFE_T_RATIO_TSP_LOW_T   = 0.90; 
    static constexpr double PENALTY_TSP_WARN_LOW_T   = 5.0;  
    static constexpr double PENALTY_TSP_FATAL_LOW_T  = 50.0; 
};

const int dr[] = {0, 1, 0, -1};
const int dc[] = {1, 0, -1, 0};

struct Ball { int sr, sc, gr, gc; };
struct Cost { int a_cost, e_cost; };
struct TaskCost { int a, e; };

struct PQNode {
    Cost cost;
    int u;
    bool operator>(const PQNode& o) const {
        if (cost.a_cost != o.cost.a_cost) return cost.a_cost > o.cost.a_cost;
        return cost.e_cost > o.cost.e_cost;
    }
};

int N, M, T;
vector<string> v_walls, h_walls;
vector<Ball> balls;

int dist_A[Config::MAX_STATES][Config::MAX_STATES];
int dist_E[Config::MAX_STATES][Config::MAX_STATES];
int prev_u[Config::MAX_STATES][Config::MAX_STATES];
char last_act[Config::MAX_STATES][Config::MAX_STATES];

Cost task_cost[Config::MAX_M][4][4];
Cost move_cost[Config::MAX_M][Config::MAX_M][4][4];
Cost start_cost[Config::MAX_M][4];
Cost combined_cost[Config::MAX_M][Config::MAX_M][4][4];
int combined_d_in[Config::MAX_M][Config::MAX_M][4][4]; 

bool can_move_arr[Config::MAX_N][Config::MAX_N][4];

int rev_P_head[Config::MAX_STATES];
int rev_P_next[Config::MAX_STATES];
int rev_P_node[Config::MAX_STATES];
int rev_P_cnt = 0;
int fwd_P[Config::MAX_STATES];

bool g_is_low_t = false; // Tの下振れ判定フラグ

inline int get_id(int r, int c, int d) { return (r * N + c) * 4 + d; }

void init_board() {
    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            can_move_arr[r][c][0] = (c + 1 < N && v_walls[r][c] == '0');
            can_move_arr[r][c][1] = (r + 1 < N && h_walls[r][c] == '0');
            can_move_arr[r][c][2] = (c - 1 >= 0 && v_walls[r][c - 1] == '0');
            can_move_arr[r][c][3] = (r - 1 >= 0 && h_walls[r - 1][c] == '0');
        }
    }
}

// -----------------------------------------------------------
// Tの下振れ判定 (数学的逆算)
// -----------------------------------------------------------
bool check_if_low_t(int t) {
    auto get_2d_dist = [&](int sr, int sc, int gr, int gc) {
        if (sr == gr && sc == gc) return 0;
        int dist[Config::MAX_N][Config::MAX_N];
        for (int i = 0; i < N; ++i) fill(dist[i], dist[i] + N, Config::INF);
        dist[sr][sc] = 0;
        
        static int qr[400], qc[400];
        int head = 0, tail = 0;
        qr[tail] = sr; qc[tail++] = sc;
        
        while (head < tail) {
            int r = qr[head], c = qc[head++];
            if (r == gr && c == gc) return dist[r][c];
            for (int d = 0; d < 4; ++d) {
                if (can_move_arr[r][c][d]) {
                    int nr = r + dr[d], nc = c + dc[d];
                    if (dist[nr][nc] == Config::INF) {
                        dist[nr][nc] = dist[r][c] + 1;
                        qr[tail] = nr; qc[tail++] = nc;
                    }
                }
            }
        }
        return Config::INF;
    };

    int X = 0;
    int curr_r = 0, curr_c = 0;
    for (int i = 0; i < M; ++i) {
        X += get_2d_dist(curr_r, curr_c, balls[i].sr, balls[i].sc);
        X += get_2d_dist(balls[i].sr, balls[i].sc, balls[i].gr, balls[i].gc);
        curr_r = balls[i].gr;
        curr_c = balls[i].gc;
    }

    double A = 2.0 * X + 4.0 * M;
    double B = 2.0 * N * N * M;
    
    // r >= 0.95 となる確率が5%（下位5%の厳しいT）
    double threshold_T = pow(A, 0.95) * pow(B, 0.05);

    cerr << "[T Gen Check] X=" << X << " | A=" << A << " | B=" << B << endl;
    cerr << "[T Gen Check] T=" << t << " | Bottom 5% Threshold=" << threshold_T << endl;

    if (t <= threshold_T) {
        cerr << "[T Gen Check] Warning: This is a LOW T (Bottom 5%) case!" << endl;
        return true;
    }
    return false;
}

int apply_macro(int u, const char* macro_str, int len) {
    int r = (u / 4) / N, c = (u / 4) % N, d = u % 4;
    for (int i = 0; i < len; ++i) {
        char op = macro_str[i];
        if (op == 'F') {
            if (can_move_arr[r][c][d]) { r += dr[d]; c += dc[d]; }
        } else if (op == 'R') { d = (d + 1) % 4; } 
        else if (op == 'L') { d = (d + 3) % 4; }
    }
    return get_id(r, c, d);
}

// -----------------------------------------------------------
// タイムスタンプO(1)配列 BFS
// -----------------------------------------------------------
int eval_dist_A[Config::MAX_STATES];
int eval_dist_E[Config::MAX_STATES];
int eval_visited[Config::MAX_STATES];
int eval_id = 0;

inline int get_A(int u) { return (eval_visited[u] == eval_id) ? eval_dist_A[u] : Config::INF; }
inline int get_E(int u) { return (eval_visited[u] == eval_id) ? eval_dist_E[u] : Config::INF; }

void run_reverse_bfs_task(int gr, int gc, int macro_e_cost) {
    eval_id++;
    static int q[65536];
    uint16_t head = 0, tail = 0;
    
    for (int d = 0; d < 4; ++d) {
        int goal_u = get_id(gr, gc, d);
        eval_visited[goal_u] = eval_id;
        eval_dist_A[goal_u] = 0;
        eval_dist_E[goal_u] = 0;
        q[tail++] = goal_u;
    }
    
    while (head != tail) {
        int u = q[head++];
        int r = (u / 4) / N, c = (u / 4) % N, d = u % 4;
        int cur_a = eval_dist_A[u], cur_e = eval_dist_E[u];

        auto try_relax = [&](int nu, int e_add) {
            int nxt_a = cur_a + 1;
            int nxt_e = cur_e + e_add;
            if (eval_visited[nu] != eval_id) {
                eval_visited[nu] = eval_id;
                eval_dist_A[nu] = nxt_a;
                eval_dist_E[nu] = nxt_e;
                q[tail++] = nu;
            } else {
                if (eval_dist_A[nu] > nxt_a) {
                    eval_dist_A[nu] = nxt_a;
                    eval_dist_E[nu] = nxt_e;
                    q[tail++] = nu;
                } else if (eval_dist_A[nu] == nxt_a && eval_dist_E[nu] > nxt_e) {
                    eval_dist_E[nu] = nxt_e;
                }
            }
        };

        int prev_r = r - dr[d], prev_c = c - dc[d];
        if (prev_r >= 0 && prev_r < N && prev_c >= 0 && prev_c < N) {
            if (can_move_arr[prev_r][prev_c][d]) try_relax(get_id(prev_r, prev_c, d), 1);
        }
        try_relax(get_id(r, c, (d + 3) % 4), 1);
        try_relax(get_id(r, c, (d + 1) % 4), 1);
        for (int e = rev_P_head[u]; e != -1; e = rev_P_next[e]) {
            try_relax(rev_P_node[e], macro_e_cost);
        }
    }
}

void run_forward_bfs(const int* start_nodes, int start_count, int macro_e_cost) {
    eval_id++;
    static int q[65536];
    uint16_t head = 0, tail = 0;
    
    for (int i = 0; i < start_count; ++i) {
        int u = start_nodes[i];
        if (eval_visited[u] != eval_id) {
            eval_visited[u] = eval_id;
            eval_dist_A[u] = 0;
            eval_dist_E[u] = 0;
            q[tail++] = u;
        }
    }
    
    while (head != tail) {
        int u = q[head++];
        int r = (u / 4) / N, c = (u / 4) % N, d = u % 4;
        int cur_a = eval_dist_A[u], cur_e = eval_dist_E[u];

        auto try_relax = [&](int nu, int e_add) {
            int nxt_a = cur_a + 1;
            int nxt_e = cur_e + e_add;
            if (eval_visited[nu] != eval_id) {
                eval_visited[nu] = eval_id;
                eval_dist_A[nu] = nxt_a;
                eval_dist_E[nu] = nxt_e;
                q[tail++] = nu;
            } else {
                if (eval_dist_A[nu] > nxt_a) {
                    eval_dist_A[nu] = nxt_a;
                    eval_dist_E[nu] = nxt_e;
                    q[tail++] = nu;
                } else if (eval_dist_A[nu] == nxt_a && eval_dist_E[nu] > nxt_e) {
                    eval_dist_E[nu] = nxt_e;
                }
            }
        };

        if (can_move_arr[r][c][d]) try_relax(get_id(r + dr[d], c + dc[d], d), 1);
        try_relax(get_id(r, c, (d + 1) % 4), 1);
        try_relax(get_id(r, c, (d + 3) % 4), 1);
        if (macro_e_cost > 0) try_relax(fwd_P[u], macro_e_cost);
    }
}

// -----------------------------------------------------------
// マクロ精密評価 (NN法近似)
// -----------------------------------------------------------
double evaluate_macro(const char* macro_buf, int macro_len) {
    fill(rev_P_head, rev_P_head + N * N * 4, -1);
    rev_P_cnt = 0;
    
    if (macro_len > 0) {
        for (int u = 0; u < N * N * 4; ++u) {
            int end_u = apply_macro(u, macro_buf, macro_len);
            fwd_P[u] = end_u;
            rev_P_node[rev_P_cnt] = u;
            rev_P_next[rev_P_cnt] = rev_P_head[end_u];
            rev_P_head[end_u] = rev_P_cnt++;
        }
    } else {
        for (int u = 0; u < N * N * 4; ++u) fwd_P[u] = u;
    }

    TaskCost c_task[Config::MAX_M];
    TaskCost c_start[Config::MAX_M];
    TaskCost c_move[Config::MAX_M][Config::MAX_M];

    for (int i = 0; i < M; ++i) {
        run_reverse_bfs_task(balls[i].gr, balls[i].gc, macro_len);
        int min_a = Config::INF, min_e = Config::INF;
        for (int d = 0; d < 4; ++d) {
            int su = get_id(balls[i].sr, balls[i].sc, d);
            int a = get_A(su), e = get_E(su);
            if (a < min_a || (a == min_a && e < min_e)) { min_a = a; min_e = e; }
        }
        c_task[i] = {min_a, min_e};
        if (c_task[i].a < Config::INF) { c_task[i].a += 2; c_task[i].e += 2; }
    }

    int zero_u = get_id(0, 0, 0);
    run_forward_bfs(&zero_u, 1, macro_len);
    for (int i = 0; i < M; ++i) {
        int min_a = Config::INF, min_e = Config::INF;
        for (int d = 0; d < 4; ++d) {
            int su = get_id(balls[i].sr, balls[i].sc, d);
            int a = get_A(su), e = get_E(su);
            if (a < min_a || (a == min_a && e < min_e)) { min_a = a; min_e = e; }
        }
        c_start[i] = {min_a, min_e};
    }

    for (int i = 0; i < M; ++i) {
        int starts[4];
        for (int d = 0; d < 4; ++d) starts[d] = get_id(balls[i].gr, balls[i].gc, d);
        run_forward_bfs(starts, 4, macro_len);
        
        for (int j = 0; j < M; ++j) {
            if (i == j) continue;
            int min_a = Config::INF, min_e = Config::INF;
            for (int d = 0; d < 4; ++d) {
                int su = get_id(balls[j].sr, balls[j].sc, d);
                int a = get_A(su), e = get_E(su);
                if (a < min_a || (a == min_a && e < min_e)) { min_a = a; min_e = e; }
            }
            c_move[i][j] = {min_a, min_e};
        }
    }

    long long total_A = 0;
    long long total_E = 0;
    bool visited[Config::MAX_M] = {false};
    int curr_pos = -1;

    for (int step = 0; step < M; ++step) {
        int best_next = -1;
        int min_A = Config::INF, min_E = Config::INF;
        for (int i = 0; i < M; ++i) {
            if (visited[i]) continue;
            int a_cost = (curr_pos == -1) ? c_start[i].a : c_move[curr_pos][i].a;
            int e_cost = (curr_pos == -1) ? c_start[i].e : c_move[curr_pos][i].e;
            if (a_cost < min_A || (a_cost == min_A && e_cost < min_E)) {
                min_A = a_cost; min_E = e_cost; best_next = i;
            }
        }
        if (best_next == -1 || min_A >= Config::INF || c_task[best_next].a >= Config::INF) {
            total_A += (M - step) * 10000; total_E += (M - step) * 10000;
            break;
        }
        total_A += min_A + c_task[best_next].a;
        total_E += min_E + c_task[best_next].e;
        visited[best_next] = true;
        curr_pos = best_next;
    }

    if (macro_len > 0) total_A += macro_len + 2; 
    total_A += M;

    double eval_score = total_A;
    eval_score += macro_len * 0.01; 
    
    int estimated_E = total_E; 
    
    // ★ フラグに応じてペナルティを切り替え
    double safe_T = g_is_low_t ? (T * Config::SAFE_T_RATIO_MACRO_LOW_T) : (T * Config::SAFE_T_RATIO_MACRO_NORMAL);
    double warn_p = g_is_low_t ? Config::PENALTY_MACRO_WARN_LOW_T : Config::PENALTY_MACRO_WARN_NORMAL;
    double fatal_p= g_is_low_t ? Config::PENALTY_MACRO_FATAL_LOW_T : Config::PENALTY_MACRO_FATAL_NORMAL;
    
    if (estimated_E > safe_T) eval_score += (estimated_E - safe_T) * warn_p; 
    if (estimated_E > T)      eval_score += (estimated_E - T) * fatal_p; 

    return eval_score;
}

int detect_w() {
    int v_count = 0, h_count = 0;
    for (int i = 0; i < N; ++i) 
        for (int j = 0; j < N - 1; ++j) 
            if (v_walls[i][j] == '1') v_count++;
            
    for (int i = 0; i < N - 1; ++i) 
        for (int j = 0; j < N; ++j) 
            if (h_walls[i][j] == '1') h_count++;
            
    if (v_count == 0 && h_count == 0) return 0;
    if (v_count > 0 && h_count == 0) {
        int col = -1;
        for (int i = 0; i < N; ++i) 
            for (int j = 0; j < N - 1; ++j) 
                if (v_walls[i][j] == '1') {
                    if (col == -1) col = j;
                    else if (col != j) return 2; 
                }
        return 1;
    } else if (h_count > 0 && v_count == 0) {
        int row = -1;
        for (int i = 0; i < N - 1; ++i) 
            for (int j = 0; j < N; ++j) 
                if (h_walls[i][j] == '1') {
                    if (row == -1) row = i;
                    else if (row != i) return 2;
                }
        return 1;
    }
    return 2;
}

struct MacroParam {
    int w_type;
    int a, b, c, d;
    int dir1, dir2;

    void get_macro(char* buf, int& len) const {
        len = 0;
        for(int i = 0; i < a; ++i) buf[len++] = 'F';
        buf[len++] = (dir1 == 0 ? 'R' : 'L');
        for(int i = 0; i < b; ++i) buf[len++] = 'F';
        
        if (w_type == 0) {
            if (c == 1) buf[len++] = 'R';
            else if (c == 2) { buf[len++] = 'R'; buf[len++] = 'R'; }
            else if (c == 3) buf[len++] = 'L';
            buf[len] = '\0';
            return;
        }

        buf[len++] = (dir2 == 0 ? 'R' : 'L');
        for(int i = 0; i < c; ++i) buf[len++] = 'F';
        
        if (d == 1) buf[len++] = 'R';
        else if (d == 2) { buf[len++] = 'R'; buf[len++] = 'R'; }
        else if (d == 3) buf[len++] = 'L';
        buf[len] = '\0';
    }

    void mutate() {
        if (w_type == 0) {
            int type = xor128() % 4;
            if (type == 0) a = (a + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 1) b = (b + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 2) c = xor128() % 4;
            else dir1 = 1 - dir1;
        } else if (w_type == 1) {
            int type = xor128() % 5;
            if (type == 0) a = (a + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 1) b = (b + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 2) c = (c + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 3) d = xor128() % 4;
            else { dir1 = 1 - dir1; dir2 = dir1; }
        } else {
            int type = xor128() % 5;
            if (type == 0) a = (a + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 1) b = (b + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 2) c = (c + (xor128() % 2 ? 1 : N - 1)) % N;
            else if (type == 3) d = xor128() % 4;
            else { dir1 = 1 - dir1; dir2 = 1 - dir1; } 
        }
    }
};

string sa_macro_search(int time_limit_ms, chrono::high_resolution_clock::time_point sa_start_time, int w, int& out_iters) {
    MacroParam curr;
    curr.w_type = min(w, 2);
    curr.a = (N / 3) + 1;
    curr.b = (N / 3) + 1;
    curr.c = (curr.w_type == 0) ? (xor128() % 4) : ((N / 3) + 1);
    curr.d = xor128() % 4;
    curr.dir1 = xor128() % 2;
    if (curr.w_type == 1) curr.dir2 = curr.dir1;
    else if (curr.w_type == 2) curr.dir2 = 1 - curr.dir1;

    char m_buf[100]; int m_len;
    curr.get_macro(m_buf, m_len);
    double current_score = evaluate_macro(m_buf, m_len);
    
    MacroParam best = curr;
    double best_score = current_score;

    double T0 = Config::MACRO_SA_T0;
    double T1 = Config::MACRO_SA_T1;
    int iter = 0;
    double Temp = T0;

    while (true) {
        if ((iter & 63) == 0) { 
            auto now = chrono::high_resolution_clock::now();
            int elapsed = chrono::duration_cast<chrono::milliseconds>(now - sa_start_time).count();
            if (elapsed >= time_limit_ms) break;
            double progress = (double)elapsed / time_limit_ms;
            Temp = T0 * pow(T1 / T0, progress); 
        }
        
        MacroParam next_param = curr;
        next_param.mutate();
        
        next_param.get_macro(m_buf, m_len);
        double next_score = evaluate_macro(m_buf, m_len);
        double diff = next_score - current_score;
        
        if (diff < 0 || exp(-diff / Temp) > (double)(xor128() % 10000) / 10000.0) {
            curr = next_param;
            current_score = next_score;
            if (current_score < best_score) {
                best_score = current_score;
                best = curr;
            }
        }
        iter++;
    }
    
    out_iters = iter;
    best.get_macro(m_buf, m_len);
    string best_macro(m_buf, m_len);
    cerr << "[SA Macro W=" << w << "] Iter: " << iter << " | Best: " << (best_macro.empty() ? "(none)" : best_macro) << endl;
    return best_macro;
}

// -----------------------------------------------------------
// 究極圧縮オプティマイザ (ウェル解析による回転吸収 + S組み込み)
// -----------------------------------------------------------

bool simulate_and_check(const string& exec_ops, const string& optimal_macro) {
    int e_cost = 0;
    int r = 0, c = 0, d = 0;
    int board_ball[Config::MAX_N][Config::MAX_N];
    for (int i = 0; i < N; ++i) for (int j = 0; j < N; ++j) board_ball[i][j] = -1;
    for (int i = 0; i < M; ++i) board_ball[balls[i].sr][balls[i].sc] = i;
    int held_ball = -1;
    
    auto process = [&](char op) {
        if (op == 'F') { if (can_move_arr[r][c][d]) { r += dr[d]; c += dc[d]; } }
        else if (op == 'R') { d = (d + 1) % 4; }
        else if (op == 'L') { d = (d + 3) % 4; }
        else if (op == 'S') { swap(held_ball, board_ball[r][c]); }
    };

    bool macro_registered = optimal_macro.empty();

    for (char op : exec_ops) {
        e_cost++;
        if (op == 'P') {
            if (!macro_registered) {
                e_cost += optimal_macro.length() + 1;
                for (char mop : optimal_macro) process(mop);
                macro_registered = true;
            } else {
                for (char mop : optimal_macro) {
                    e_cost++;
                    process(mop);
                }
            }
        } else {
            process(op);
        }
    }

    if (e_cost > T) return false;
    for (int i = 0; i < M; ++i) {
        if (board_ball[balls[i].gr][balls[i].gc] != i) return false;
    }
    return true;
}

int get_final_length(const string& exec_ops, const string& optimal_macro) {
    if (optimal_macro.empty()) return exec_ops.length();
    if (exec_ops.find('P') != string::npos) {
        return exec_ops.length() + optimal_macro.length() + 1;
    }
    return exec_ops.length();
}

void parse_wells(const string& s, vector<int>& S, vector<int>& R, vector<char>& C) {
    int cur_s = 0, cur_r = 0;
    for(char c : s) {
        if(c == 'S') cur_s ^= 1;
        else if(c == 'R') cur_r = (cur_r + 1) % 4;
        else if(c == 'L') cur_r = (cur_r + 3) % 4;
        else {
            S.push_back(cur_s); R.push_back(cur_r); C.push_back(c);
            cur_s = 0; cur_r = 0;
        }
    }
    S.push_back(cur_s); R.push_back(cur_r);
}

string build_from_wells(const vector<int>& S, const vector<int>& R, const vector<char>& C) {
    string res = "";
    for(size_t i=0; i<C.size(); ++i) {
        if(S[i]) res += "S";
        if(R[i] == 1) res += "R";
        else if(R[i] == 2) res += "RR";
        else if(R[i] == 3) res += "L";
        res += C[i];
    }
    if(S.back()) res += "S";
    if(R.back() == 1) res += "R";
    else if(R.back() == 2) res += "RR";
    else if(R.back() == 3) res += "L";
    return res;
}

void full_optimize(string& exec_ops, string& optimal_macro) {
    bool overall_changed = true;
    int loop_count = 0;
    int initial_len = get_final_length(exec_ops, optimal_macro);

    while (overall_changed && loop_count < 100) {
        overall_changed = false;
        loop_count++;
        
        bool rot_changed = true;
        while (rot_changed) {
            rot_changed = false;
            
            vector<int> S, R; vector<char> C;
            parse_wells(exec_ops, S, R, C);
            exec_ops = build_from_wells(S, R, C);
            
            vector<int> mS, mR; vector<char> mC;
            parse_wells(optimal_macro, mS, mR, mC);
            optimal_macro = build_from_wells(mS, mR, mC);

            int current_len = get_final_length(exec_ops, optimal_macro);
            int best_diff = 0;
            int best_action = -1;

            for (int act = 0; act < 4; ++act) {
                if (optimal_macro.empty() && count(C.begin(), C.end(), 'P') == 0) continue;

                vector<int> nR = R;
                string n_mac = optimal_macro;

                if (act == 0) { 
                    n_mac = "L" + n_mac;
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') nR[i] = (nR[i] + 1) % 4;
                } else if (act == 1) { 
                    n_mac = "R" + n_mac;
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') nR[i] = (nR[i] + 3) % 4;
                } else if (act == 2) { 
                    n_mac = n_mac + "L";
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') nR[i+1] = (nR[i+1] + 1) % 4;
                } else if (act == 3) { 
                    n_mac = n_mac + "R";
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') nR[i+1] = (nR[i+1] + 3) % 4;
                }

                string n_exec = build_from_wells(S, nR, C);
                vector<int> tmS, tmR; vector<char> tmC;
                parse_wells(n_mac, tmS, tmR, tmC);
                n_mac = build_from_wells(tmS, tmR, tmC);

                int new_len = get_final_length(n_exec, n_mac);
                int diff = current_len - new_len;

                if (diff > best_diff && simulate_and_check(n_exec, n_mac)) {
                    best_diff = diff;
                    best_action = act;
                }
            }

            if (best_action != -1) {
                if (best_action == 0) {
                    optimal_macro = "L" + optimal_macro;
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') R[i] = (R[i] + 1) % 4;
                } else if (best_action == 1) {
                    optimal_macro = "R" + optimal_macro;
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') R[i] = (R[i] + 3) % 4;
                } else if (best_action == 2) {
                    optimal_macro = optimal_macro + "L";
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') R[i+1] = (R[i+1] + 1) % 4;
                } else if (best_action == 3) {
                    optimal_macro = optimal_macro + "R";
                    for(size_t i=0; i<C.size(); ++i) if(C[i] == 'P') R[i+1] = (R[i+1] + 3) % 4;
                }
                
                exec_ops = build_from_wells(S, R, C);
                vector<int> tmS, tmR; vector<char> tmC;
                parse_wells(optimal_macro, tmS, tmR, tmC);
                optimal_macro = build_from_wells(tmS, tmR, tmC);
                
                rot_changed = true;
                overall_changed = true;
            }
        }

        if (optimal_macro.empty()) continue;

        int current_len = get_final_length(exec_ops, optimal_macro);
        
        int r_old = 0, c_old = 0, d_old = 0;
        int held_old = -1;
        int board_old[Config::MAX_N][Config::MAX_N];
        for (int i = 0; i < N; ++i) for (int j = 0; j < N; ++j) board_old[i][j] = -1;
        for (int i = 0; i < M; ++i) board_old[balls[i].sr][balls[i].sc] = i;

        auto process_old = [&](char op) {
            if (op == 'F') { if (can_move_arr[r_old][c_old][d_old]) { r_old += dr[d_old]; c_old += dc[d_old]; } }
            else if (op == 'R') { d_old = (d_old + 1) % 4; }
            else if (op == 'L') { d_old = (d_old + 3) % 4; }
            else if (op == 'S') { swap(held_old, board_old[r_old][c_old]); }
        };
        
        string n_mac_s = "S" + optimal_macro;
        string n_exec_s = "";
        
        for (size_t i = 0; i < exec_ops.length(); ++i) {
            if (exec_ops[i] == 'S' && i + 1 < exec_ops.length() && exec_ops[i+1] == 'P') {
                n_exec_s += "P";
                process_old('S'); 
                for (char mop : optimal_macro) process_old(mop);
                i++; 
            } else if (exec_ops[i] == 'P') {
                if (held_old != -1 || board_old[r_old][c_old] != -1) {
                    n_exec_s += "SP";
                } else {
                    n_exec_s += "P"; 
                }
                for (char mop : optimal_macro) process_old(mop);
            } else {
                n_exec_s += exec_ops[i];
                process_old(exec_ops[i]);
            }
        }
        
        vector<int> sS, sR; vector<char> sC;
        parse_wells(n_exec_s, sS, sR, sC);
        n_exec_s = build_from_wells(sS, sR, sC);
        
        vector<int> smS, smR; vector<char> smC;
        parse_wells(n_mac_s, smS, smR, smC);
        n_mac_s = build_from_wells(smS, smR, smC);

        int new_len = get_final_length(n_exec_s, n_mac_s);
        
        if (new_len < current_len && simulate_and_check(n_exec_s, n_mac_s)) {
            exec_ops = n_exec_s;
            optimal_macro = n_mac_s;
            overall_changed = true; 
        }
    }

    int final_len = get_final_length(exec_ops, optimal_macro);
    if (final_len < initial_len) {
        cerr << "[Optimize] Compressed " << initial_len << " -> " << final_len << " ops recursively!" << endl;
    }
}

// -----------------------------------------------------------
// TSP準備と実行
// -----------------------------------------------------------
void precompute_dijkstra(const char* macro_str, int macro_len, int initial_start_u) {
    int num_states = N * N * 4;

    vector<int> src_list;
    src_list.push_back(initial_start_u);
    for (int i = 0; i < M; ++i) {
        for (int d = 0; d < 4; ++d) {
            src_list.push_back(get_id(balls[i].sr, balls[i].sc, d));
            src_list.push_back(get_id(balls[i].gr, balls[i].gc, d));
        }
    }
    sort(src_list.begin(), src_list.end());
    src_list.erase(unique(src_list.begin(), src_list.end()), src_list.end());

    for (int i : src_list) {
        for (int j = 0; j < num_states; ++j) {
            dist_A[i][j] = Config::INF;
            dist_E[i][j] = Config::INF;
        }
        dist_A[i][i] = 0;
        dist_E[i][i] = 0;

        priority_queue<PQNode, vector<PQNode>, greater<PQNode>> pq;
        pq.push({{0, 0}, i});
        
        while (!pq.empty()) {
            PQNode cur = pq.top();
            pq.pop();
            
            Cost c_cost = cur.cost;
            int u = cur.u;
            
            if (c_cost.a_cost > dist_A[i][u] || (c_cost.a_cost == dist_A[i][u] && c_cost.e_cost > dist_E[i][u])) continue;
            
            int r = (u / 4) / N, c = (u / 4) % N, d = u % 4;

            auto try_relax = [&](int nu, char act, int e_add) {
                Cost nxt_cost = {c_cost.a_cost + 1, c_cost.e_cost + e_add};
                if (dist_A[i][nu] > nxt_cost.a_cost || 
                   (dist_A[i][nu] == nxt_cost.a_cost && dist_E[i][nu] > nxt_cost.e_cost)) {
                    dist_A[i][nu] = nxt_cost.a_cost;
                    dist_E[i][nu] = nxt_cost.e_cost;
                    prev_u[i][nu] = u;
                    last_act[i][nu] = act;
                    pq.push({nxt_cost, nu});
                }
            };
            
            if (can_move_arr[r][c][d]) try_relax(get_id(r + dr[d], c + dc[d], d), 'F', 1);
            try_relax(get_id(r, c, (d + 1) % 4), 'R', 1);
            try_relax(get_id(r, c, (d + 3) % 4), 'L', 1);
            if (macro_len > 0) try_relax(apply_macro(u, macro_str, macro_len), 'P', macro_len);
        }
    }
}

void precompute_tsp_costs(int start_u) {
    for (int i = 0; i < M; ++i) {
        for (int d_in = 0; d_in < 4; ++d_in) {
            int s_u = get_id(balls[i].sr, balls[i].sc, d_in);
            start_cost[i][d_in] = {dist_A[start_u][s_u], dist_E[start_u][s_u]};
            for (int d_out = 0; d_out < 4; ++d_out) {
                int g_u = get_id(balls[i].gr, balls[i].gc, d_out);
                task_cost[i][d_in][d_out] = {dist_A[s_u][g_u] + 2, dist_E[s_u][g_u] + 2};
            }
        }
    }
    for (int i = 0; i < M; ++i) {
        for (int j = 0; j < M; ++j) {
            if (i == j) continue;
            for (int d_out = 0; d_out < 4; ++d_out) {
                int g_u = get_id(balls[i].gr, balls[i].gc, d_out);
                for (int d_in = 0; d_in < 4; ++d_in) {
                    int s_u = get_id(balls[j].sr, balls[j].sc, d_in);
                    move_cost[i][j][d_out][d_in] = {dist_A[g_u][s_u], dist_E[g_u][s_u]};
                }
            }
        }
    }
    for (int i = 0; i < M; ++i) {
        for (int j = 0; j < M; ++j) {
            if (i == j) continue;
            for (int pd_out = 0; pd_out < 4; ++pd_out) {
                for (int d_out = 0; d_out < 4; ++d_out) {
                    combined_cost[i][j][pd_out][d_out] = {Config::INF, Config::INF};
                    for (int d_in = 0; d_in < 4; ++d_in) {
                        int a = move_cost[i][j][pd_out][d_in].a_cost + task_cost[j][d_in][d_out].a_cost;
                        int e = move_cost[i][j][pd_out][d_in].e_cost + task_cost[j][d_in][d_out].e_cost;
                        if (a < combined_cost[i][j][pd_out][d_out].a_cost || 
                           (a == combined_cost[i][j][pd_out][d_out].a_cost && e < combined_cost[i][j][pd_out][d_out].e_cost)) {
                            combined_cost[i][j][pd_out][d_out] = {a, e};
                            combined_d_in[i][j][pd_out][d_out] = d_in;
                        }
                    }
                }
            }
        }
    }
}

struct TSPState {
    int p[Config::MAX_M];
    Cost dp[Config::MAX_M][4];
    int prev_d_out[Config::MAX_M][4]; 
    int prev_d_in[Config::MAX_M][4];
    int total_A, total_E;

    void evaluate(int start_idx, int initial_a_cost, int initial_e_cost) {
        double safe_T_ratio = g_is_low_t ? Config::SAFE_T_RATIO_TSP_LOW_T : Config::SAFE_T_RATIO_TSP_NORMAL;
        double warn_p       = g_is_low_t ? Config::PENALTY_TSP_WARN_LOW_T : Config::PENALTY_TSP_WARN_NORMAL;
        double fatal_p      = g_is_low_t ? Config::PENALTY_TSP_FATAL_LOW_T : Config::PENALTY_TSP_FATAL_NORMAL;

        if (start_idx == 0) {
            for (int d_out = 0; d_out < 4; ++d_out) {
                dp[0][d_out] = {Config::INF, Config::INF};
                for (int d_in = 0; d_in < 4; ++d_in) {
                    int a = initial_a_cost + start_cost[p[0]][d_in].a_cost + task_cost[p[0]][d_in][d_out].a_cost;
                    int e = initial_e_cost + start_cost[p[0]][d_in].e_cost + task_cost[p[0]][d_in][d_out].e_cost;
                    if (a < dp[0][d_out].a_cost || (a == dp[0][d_out].a_cost && e < dp[0][d_out].e_cost)) {
                        dp[0][d_out] = {a, e};
                        prev_d_in[0][d_out] = d_in;
                    }
                }
            }
            start_idx = 1;
        }

        for (int i = start_idx; i < M; ++i) {
            for (int d = 0; d < 4; ++d) dp[i][d] = {Config::INF, Config::INF};
            for (int d_out = 0; d_out < 4; ++d_out) {
                for (int pd_out = 0; pd_out < 4; ++pd_out) {
                    if (dp[i-1][pd_out].a_cost >= Config::INF) continue;
                    
                    int a = dp[i-1][pd_out].a_cost + combined_cost[p[i-1]][p[i]][pd_out][d_out].a_cost;
                    int e = dp[i-1][pd_out].e_cost + combined_cost[p[i-1]][p[i]][pd_out][d_out].e_cost;
                    
                    int penalty = 0;
                    if (e > T) penalty = (e - T) * (e - T) * fatal_p; 
                    else if (e > T * safe_T_ratio) penalty = (e - (int)(T * safe_T_ratio)) * warn_p;
                    a += penalty;

                    if (a < dp[i][d_out].a_cost || (a == dp[i][d_out].a_cost && e < dp[i][d_out].e_cost)) {
                        dp[i][d_out] = {a, e};
                        prev_d_out[i][d_out] = pd_out;
                        prev_d_in[i][d_out] = combined_d_in[p[i-1]][p[i]][pd_out][d_out];
                    }
                }
            }
        }

        int best_last_d = 0;
        for (int d = 1; d < 4; ++d) {
            if (dp[M-1][d].a_cost < dp[M-1][best_last_d].a_cost || 
               (dp[M-1][d].a_cost == dp[M-1][best_last_d].a_cost && dp[M-1][d].e_cost < dp[M-1][best_last_d].e_cost)) {
                best_last_d = d;
            }
        }
        total_A = dp[M-1][best_last_d].a_cost;
        total_E = dp[M-1][best_last_d].e_cost;
    }

    void extract_result(vector<int>& out_d_in, vector<int>& out_d_out) const {
        int best_last_d = 0;
        for (int d = 1; d < 4; ++d) {
            if (dp[M-1][d].a_cost < dp[M-1][best_last_d].a_cost || 
               (dp[M-1][d].a_cost == dp[M-1][best_last_d].a_cost && dp[M-1][d].e_cost < dp[M-1][best_last_d].e_cost)) {
                best_last_d = d;
            }
        }
        out_d_in.resize(M);
        out_d_out.resize(M);
        int curr_d = best_last_d;
        for (int i = M - 1; i >= 0; --i) {
            out_d_out[i] = curr_d;
            out_d_in[i] = prev_d_in[i][curr_d];
            if (i > 0) curr_d = prev_d_out[i][curr_d];
        }
    }
};

TSPState sa_tsp_search(int initial_a_cost, int initial_e_cost, int time_limit_ms, chrono::high_resolution_clock::time_point tsp_start_time, int& out_iters) {
    TSPState best_initial_state;
    best_initial_state.total_A = Config::INF;
    best_initial_state.total_E = Config::INF;

    for (int start_idx = 0; start_idx < M; ++start_idx) {
        TSPState state;
        state.p[0] = start_idx;
        int idx = 1;
        for (int i = 0; i < M; ++i) if (i != start_idx) state.p[idx++] = i;

        for(int i = 0; i < M - 1; ++i) {
            int best_j = i + 1, min_dist = Config::INF;
            for(int j = i + 1; j < M; ++j) {
                int d = Config::INF;
                for(int d_out = 0; d_out < 4; ++d_out) {
                    for(int d_in = 0; d_in < 4; ++d_in) {
                        d = min(d, move_cost[state.p[i]][state.p[j]][d_out][d_in].a_cost);
                    }
                }
                if(d < min_dist) { min_dist = d; best_j = j; }
            }
            swap(state.p[i+1], state.p[best_j]);
        }
        
        state.evaluate(0, initial_a_cost, initial_e_cost);
        if (state.total_A < best_initial_state.total_A || 
           (state.total_A == best_initial_state.total_A && state.total_E < best_initial_state.total_E)) {
            best_initial_state = state;
        }
    }

    TSPState current_state = best_initial_state;
    TSPState best_state = best_initial_state;

    double T0 = Config::TSP_SA_T0;
    double T1 = Config::TSP_SA_T1;
    int iter = 0;
    double Temp = T0;

    while (true) {
        if ((iter & 127) == 0) {
            auto now = chrono::high_resolution_clock::now();
            int elapsed = chrono::duration_cast<chrono::milliseconds>(now - tsp_start_time).count();
            if (elapsed >= time_limit_ms) break; 
            double progress = (double)elapsed / time_limit_ms;
            Temp = T0 * pow(T1 / T0, progress);
        }

        TSPState next_state = current_state;
        int type = xor128() % 4; 
        int diff_idx = 0;

        if (type == 0) { 
            int i = xor128() % M, j = xor128() % M;
            swap(next_state.p[i], next_state.p[j]);
            diff_idx = min(i, j);
        } else if (type == 1) { 
            int i = xor128() % M, j = xor128() % M;
            int val = next_state.p[i];
            for(int k=i; k<M-1; ++k) next_state.p[k] = next_state.p[k+1];
            for(int k=M-1; k>j; --k) next_state.p[k] = next_state.p[k-1];
            next_state.p[j] = val;
            diff_idx = min(i, j);
        } else if (type == 2) { 
            int i = xor128() % M, j = xor128() % M;
            if (i > j) swap(i, j);
            reverse(next_state.p + i, next_state.p + j + 1);
            diff_idx = i;
        } else {
            int len = 2 + (xor128() % 3); 
            if (len >= M) len = M - 1;
            int i = xor128() % (M - len + 1); 
            int block[3];
            for(int k=0; k<len; ++k) block[k] = next_state.p[i+k];
            for(int k=i; k<M-len; ++k) next_state.p[k] = next_state.p[k+len];
            int j = xor128() % (M - len + 1); 
            for(int k=M-1; k>=j+len; --k) next_state.p[k] = next_state.p[k-len];
            for(int k=0; k<len; ++k) next_state.p[j+k] = block[k];
            diff_idx = min(i, j);
        }

        next_state.evaluate(diff_idx, initial_a_cost, initial_e_cost);
        double diff = next_state.total_A - current_state.total_A;

        if (diff < 0 || exp(-diff / Temp) > (double)(xor128() % 10000) / 10000.0) {
            current_state = next_state;
            if (current_state.total_A < best_state.total_A || 
               (current_state.total_A == best_state.total_A && current_state.total_E < best_state.total_E)) {
                best_state = current_state;
            }
        }
        iter++;
    }

    out_iters = iter;
    cerr << "[SA TSP] Iter: " << iter << " | Final Ops(A): " << best_state.total_A << " | Cost(E): " << best_state.total_E << endl;
    return best_state;
}

string get_path(int start, int goal) {
    if (start == goal) return "";
    string path = "";
    int curr = goal;
    while (curr != start) {
        path += last_act[start][curr];
        curr = prev_u[start][curr];
    }
    reverse(path.begin(), path.end());
    return path;
}

// -----------------------------------------------------------
// 貪欲フォールバック (マクロなし)
// -----------------------------------------------------------
void execute_greedy_fallback() {
    cerr << "[Warning] Switching to Greedy without macro." << endl;
    
    int start_u = get_id(0, 0, 0);
    precompute_dijkstra("", 0, start_u);
    
    string greedy_ops = "";
    int curr_u = start_u;
    bool visited[Config::MAX_M] = {false};
    
    for (int step = 0; step < M; ++step) {
        int best_ball = -1;
        int best_d_in = -1;
        int min_cost = Config::INF;
        
        for (int i = 0; i < M; ++i) {
            if (visited[i]) continue;
            for (int d = 0; d < 4; ++d) {
                int s_u = get_id(balls[i].sr, balls[i].sc, d);
                if (dist_A[curr_u][s_u] < min_cost) {
                    min_cost = dist_A[curr_u][s_u];
                    best_ball = i;
                    best_d_in = d;
                }
            }
        }
        
        int b_start_u = get_id(balls[best_ball].sr, balls[best_ball].sc, best_d_in);
        greedy_ops += get_path(curr_u, b_start_u);
        greedy_ops += "S";
        
        int best_d_out = -1;
        int min_goal_cost = Config::INF;
        for (int d = 0; d < 4; ++d) {
            int g_u = get_id(balls[best_ball].gr, balls[best_ball].gc, d);
            if (dist_A[b_start_u][g_u] < min_goal_cost) {
                min_goal_cost = dist_A[b_start_u][g_u];
                best_d_out = d;
            }
        }
        
        int b_goal_u = get_id(balls[best_ball].gr, balls[best_ball].gc, best_d_out);
        greedy_ops += get_path(b_start_u, b_goal_u);
        greedy_ops += "S";
        
        curr_u = b_goal_u;
        visited[best_ball] = true;
    }
    
    if (greedy_ops.length() > (size_t)T) greedy_ops = greedy_ops.substr(0, T);
    
    cerr << "[Greedy Fallback] Score: " << greedy_ops.length() << endl;
    for (char op : greedy_ops) cout << op << "\n";
}

// -----------------------------------------------------------
// メイン処理
// -----------------------------------------------------------
int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    init_rand();
    auto total_start_time = chrono::high_resolution_clock::now();
    int total_time_limit_ms = 1950; 

    if (!(cin >> N >> M >> T)) return 0;

    v_walls.resize(N);
    for (int i = 0; i < N; ++i) cin >> v_walls[i];

    h_walls.resize(N - 1);
    for (int i = 0; i < N - 1; ++i) cin >> h_walls[i];

    balls.resize(M);
    for (int i = 0; i < M; ++i) {
        cin >> balls[i].sr >> balls[i].sc >> balls[i].gr >> balls[i].gc;
    }

    init_board();

    int w = detect_w();
    g_is_low_t = check_if_low_t(T);

    int macro_time_limit = total_time_limit_ms - Config::TSP_MIN_TIME_MS - Config::TIME_MARGIN_MS; 

    int macro_iters = 0;
    string optimal_macro = sa_macro_search(macro_time_limit, total_start_time, w, macro_iters);
    
    int start_u = get_id(0, 0, 0); 

    precompute_dijkstra(optimal_macro.c_str(), optimal_macro.length(), start_u);
    precompute_tsp_costs(start_u);

    auto now = chrono::high_resolution_clock::now();
    int elapsed_so_far = chrono::duration_cast<chrono::milliseconds>(now - total_start_time).count();
    
    int actual_tsp_time = max(0, total_time_limit_ms - elapsed_so_far - (Config::TIME_MARGIN_MS / 2));

    int tsp_iters = 0;
    int initial_a_cost = optimal_macro.empty() ? 0 : optimal_macro.length() + 2; 
    int initial_e_cost = optimal_macro.empty() ? 0 : optimal_macro.length();
    TSPState best_state = sa_tsp_search(initial_a_cost, initial_e_cost, actual_tsp_time, now, tsp_iters);

    // ★ Tを超過した場合は安全な貪欲に切り替え
    if (best_state.total_E > T) {
        execute_greedy_fallback();
        return 0;
    }

    vector<int> d_in, d_out;
    best_state.extract_result(d_in, d_out);

    string exec_ops = "";
    int curr_u = start_u;

    for (int i = 0; i < M; ++i) {
        int b_idx = best_state.p[i];
        int b_d_in = d_in[i];
        int b_d_out = d_out[i];
        
        int b_start_u = get_id(balls[b_idx].sr, balls[b_idx].sc, b_d_in);
        int b_goal_u = get_id(balls[b_idx].gr, balls[b_idx].gc, b_d_out);
        
        exec_ops += get_path(curr_u, b_start_u);
        exec_ops += "S";
        exec_ops += get_path(b_start_u, b_goal_u);
        exec_ops += "S";
        
        curr_u = b_goal_u;
    }
    
    full_optimize(exec_ops, optimal_macro);

    string final_ops = "";
    if (optimal_macro.empty() || exec_ops.find('P') == string::npos) {
        final_ops = exec_ops;
    } else {
        bool first_P = true;
        for (char op : exec_ops) {
            if (op == 'P' && first_P) {
                final_ops += "M" + optimal_macro + "M";
                first_P = false;
            } else {
                final_ops += op;
            }
        }
    }

    if (final_ops.length() > (size_t)T) final_ops = final_ops.substr(0, T);
    
    int count_P = 0;
    for (char c : final_ops) if (c == 'P') count_P++;

    auto total_end_time = chrono::high_resolution_clock::now();
    cerr << "[Total Time] " << chrono::duration_cast<chrono::milliseconds>(total_end_time - total_start_time).count() << "ms" << endl;
    cerr << "[Iterations] Macro: " << macro_iters << " | TSP: " << tsp_iters << endl;
    cerr << "Score: " << final_ops.length() << endl;

    for (char op : final_ops) cout << op << "\n";

    return 0;
}