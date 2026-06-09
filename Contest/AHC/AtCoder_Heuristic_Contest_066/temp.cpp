#pragma GCC optimize("O3")
#pragma GCC optimize("unroll-loops")

#include <iostream>
#include <vector>
#include <string>
#include <algorithm>
#include <chrono>
#include <queue>

using namespace std;
using namespace std::chrono;

const int dr[] = {0, 1, 0, -1}; // 0:右, 1:下, 2:左, 3:上
const int dc[] = {1, 0, -1, 0};

const char OPS[] = {'F', 'R', 'L', 'S'};

// 🎯 最大長は25のままで、AIに自動調整させます
const int MAX_MACRO_LEN = 25;   

int N, M, T;

bool wall_v[25][25];
bool wall_h[25][25];
int dist_ball[45][25][25];
int dist_basket[45][25][25];

struct Pos { int r, c; };
Pos balls[45], baskets[45];

uint64_t z_pos[25][25];
uint64_t z_dir[4];
uint64_t z_hold[46]; 
uint64_t z_comp[45];
uint64_t z_is_rec[2];

uint64_t rnd() {
    static uint64_t x = 88172645463325252ULL;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    return x;
}

void init_zobrist() {
    for(int i=0; i<25; ++i) for(int j=0; j<25; ++j) z_pos[i][j] = rnd();
    for(int i=0; i<4; ++i) z_dir[i] = rnd();
    for(int i=0; i<46; ++i) z_hold[i] = rnd();
    for(int i=0; i<45; ++i) z_comp[i] = rnd();
    z_is_rec[0] = rnd();
    z_is_rec[1] = rnd();
}

inline uint64_t append_op(uint64_t macro, int op_idx) {
    uint64_t len = macro & 0x1F;
    uint64_t seq = macro >> 5;
    seq |= ((uint64_t)op_idx << (len * 2));
    return (seq << 5) | (len + 1);
}

inline uint64_t append_macro(uint64_t dest, uint64_t src) {
    uint64_t dest_len = dest & 0x1F;
    uint64_t src_len = src & 0x1F;
    uint64_t dest_seq = dest >> 5;
    uint64_t src_seq = src >> 5;
    dest_seq |= (src_seq << (dest_len * 2));
    return (dest_seq << 5) | (dest_len + src_len);
}

struct HistoryNode {
    int parent_idx;
    char op;
};
vector<HistoryNode> history;

struct State {
    uint64_t completed;
    uint64_t reg_macro, rec_macro; 
    int score;
    int history_idx;
    int real_cost; 
    int8_t r, c, dir, holding;
    bool is_rec;
    char temp_op; 
    uint64_t z_hash; 
};

const int HASH_SIZE = 1 << 22; 
uint64_t hash_key[HASH_SIZE];
int hash_step[HASH_SIZE];      
int hash_real_cost[HASH_SIZE]; 
int hash_beam_idx[HASH_SIZE]; 

int run_id = 0;

void precompute_distances() {
    for(int k=0; k<M; ++k) {
        for(int i=0; i<N; ++i) for(int j=0; j<N; ++j) {
            dist_ball[k][i][j] = 1e9;
            dist_basket[k][i][j] = 1e9;
        }
    }
    auto bfs = [&](int sr, int sc, int dist_map[25][25]) {
        queue<pair<int, int>> q;
        q.push({sr, sc});
        dist_map[sr][sc] = 0;
        while (!q.empty()) {
            auto [r, c] = q.front(); q.pop();
            for (int d = 0; d < 4; ++d) {
                int nr = r + dr[d], nc = c + dc[d];
                if (nr < 0 || nr >= N || nc < 0 || nc >= N) continue;
                if (d == 0 && wall_v[r][c]) continue;
                if (d == 1 && wall_h[r][c]) continue;
                if (d == 2 && wall_v[r][c - 1]) continue;
                if (d == 3 && wall_h[r - 1][c]) continue;
                
                if (dist_map[nr][nc] > dist_map[r][c] + 1) {
                    dist_map[nr][nc] = dist_map[r][c] + 1;
                    q.push({nr, nc});
                }
            }
        }
    };
    for (int k = 0; k < M; ++k) {
        bfs(balls[k].r, balls[k].c, dist_ball[k]);
        bfs(baskets[k].r, baskets[k].c, dist_basket[k]);
    }
}

string solve_greedy() {
    string ops = "";
    int r = 0, c = 0, dir = 0;
    int holding = -1;
    uint64_t completed = 0;
    int real_cost = 0;
    
    auto apply_F = [&]() {
        int nr = r + dr[dir], nc = c + dc[dir];
        bool hit = false;
        if (nr < 0 || nr >= N || nc < 0 || nc >= N) hit = true;
        else if (dir == 0 && wall_v[r][c]) hit = true;
        else if (dir == 1 && wall_h[r][c]) hit = true;
        else if (dir == 2 && wall_v[r][c - 1]) hit = true;
        else if (dir == 3 && wall_h[r - 1][c]) hit = true;
        if (!hit) { r = nr; c = nc; }
        real_cost++;
    };

    ops += 'M';
    for(int i = 0; i < 4; i++) { ops += 'F'; apply_F(); }
    ops += 'M';

    while (completed != ((1ULL << M) - 1)) {
        if (real_cost > T || ops.length() > T) break;

        int target_r = -1, target_c = -1, target_k = -1;
        if (holding == -1) {
            int min_d = 1e9;
            for (int k = 0; k < M; ++k) {
                if (!(completed & (1ULL << k))) {
                    if (dist_ball[k][r][c] < min_d) {
                        min_d = dist_ball[k][r][c];
                        target_k = k;
                    }
                }
            }
            if (target_k == -1) break;
            target_r = balls[target_k].r;
            target_c = balls[target_k].c;
        } else {
            target_r = baskets[holding].r;
            target_c = baskets[holding].c;
        }

        while (r != target_r || c != target_c) {
            if (real_cost > T || ops.length() > T) break;
            
            int best_d = -1;
            int current_dist = (holding == -1) ? dist_ball[target_k][r][c] : dist_basket[holding][r][c];
            
            for (int d = 0; d < 4; ++d) {
                int nr = r + dr[d], nc = c + dc[d];
                bool hit = false;
                if (nr < 0 || nr >= N || nc < 0 || nc >= N) hit = true;
                else if (d == 0 && wall_v[r][c]) hit = true;
                else if (d == 1 && wall_h[r][c]) hit = true;
                else if (d == 2 && wall_v[r][c - 1]) hit = true;
                else if (d == 3 && wall_h[r - 1][c]) hit = true;
                
                if (!hit) {
                    int next_dist = (holding == -1) ? dist_ball[target_k][nr][nc] : dist_basket[holding][nr][nc];
                    if (next_dist < current_dist) {
                        best_d = d;
                        break;
                    }
                }
            }
            if (best_d == -1) break; 

            if (dir != best_d) {
                int diff = (best_d - dir + 4) % 4;
                if (diff == 1) { ops += 'R'; dir = (dir + 1) % 4; real_cost++; }
                else if (diff == 3) { ops += 'L'; dir = (dir + 3) % 4; real_cost++; }
                else { ops += "RR"; dir = (dir + 2) % 4; real_cost+=2; }
                continue;
            }

            int straight_len = 0;
            int tr = r, tc = c;
            while (true) {
                int nr = tr + dr[dir], nc = tc + dc[dir];
                bool hit = false;
                if (nr < 0 || nr >= N || nc < 0 || nc >= N) hit = true;
                else if (dir == 0 && wall_v[tr][tc]) hit = true;
                else if (dir == 1 && wall_h[tr][tc]) hit = true;
                else if (dir == 2 && wall_v[tr][tc - 1]) hit = true;
                else if (dir == 3 && wall_h[tr - 1][tc]) hit = true;
                
                if (hit) break;
                
                int next_dist = (holding == -1) ? dist_ball[target_k][nr][nc] : dist_basket[holding][nr][nc];
                int curr_dist = (holding == -1) ? dist_ball[target_k][tr][tc] : dist_basket[holding][tr][tc];
                if (next_dist >= curr_dist) break; 
                
                straight_len++;
                tr = nr; tc = nc;
            }

            if (straight_len >= 4) {
                ops += 'P';
                for (int i = 0; i < 4; i++) apply_F(); 
            } else {
                ops += 'F';
                apply_F();
            }
        }

        if (r == target_r && c == target_c) {
            ops += 'S';
            real_cost++;
            if (holding == -1) holding = target_k;
            else {
                completed |= (1ULL << holding);
                holding = -1;
            }
        }
    }
    
    if (ops.length() > T) ops = ops.substr(0, T);
    return ops;
}

inline int evaluate(int r, int c, int holding, uint64_t completed, bool is_rec, int step) {
    int score = __builtin_popcountll(completed) * 100000;
    if (holding != -1) {
        score += 50000; 
        score -= dist_basket[holding][r][c] * 2; 
    } else {
        int min_d = 1e9;
        for (int k = 0; k < M; ++k) {
            if (!(completed & (1ULL << k))) {
                if (dist_ball[k][r][c] < min_d) min_d = dist_ball[k][r][c];
            }
        }
        if (min_d != 1e9) score -= min_d * 2; 
    }
    if (is_rec) score -= 2; 
    
    return score - step; 
}

inline bool sim_base(int8_t &r, int8_t &c, int8_t &dir, int8_t &holding, uint64_t &completed, int op_idx, uint64_t &z_hash) {
    if (op_idx == 0) {
        int nr = r + dr[dir], nc = c + dc[dir];
        bool hit = false;
        if (nr < 0 || nr >= N || nc < 0 || nc >= N) hit = true;
        else if (dir == 0 && wall_v[r][c]) hit = true;
        else if (dir == 1 && wall_h[r][c]) hit = true;
        else if (dir == 2 && wall_v[r][c - 1]) hit = true;
        else if (dir == 3 && wall_h[r - 1][c]) hit = true;
        if (!hit) { 
            z_hash ^= z_pos[r][c] ^ z_pos[nr][nc];
            r = nr; c = nc; 
        }
        return true;
    } else if (op_idx == 1) {
        z_hash ^= z_dir[dir];
        dir = (dir + 1) % 4; 
        z_hash ^= z_dir[dir];
        return true;
    } else if (op_idx == 2) {
        z_hash ^= z_dir[dir];
        dir = (dir + 3) % 4; 
        z_hash ^= z_dir[dir];
        return true;
    } else {
        if (holding == -1) {
            for (int k = 0; k < M; ++k) {
                if (!(completed & (1ULL << k)) && r == balls[k].r && c == balls[k].c) {
                    z_hash ^= z_hold[45] ^ z_hold[k]; 
                    holding = k; return true;
                }
                if ((completed & (1ULL << k)) && r == baskets[k].r && c == baskets[k].c) {
                    return false; 
                }
            }
            return true; 
        } else {
            if (r == baskets[holding].r && c == baskets[holding].c) {
                z_hash ^= z_hold[holding] ^ z_hold[45];
                z_hash ^= z_comp[holding]; 
                completed |= (1ULL << holding);
                holding = -1;
                return true;
            }
            return false; 
        }
    }
}

int main() {
    ios_base::sync_with_stdio(false); cin.tie(NULL);
    if (!(cin >> N >> M >> T)) return 0;
    auto start_time = high_resolution_clock::now();
    double TIME_LIMIT = 1.95; 

    init_zobrist();

    for(int i=0; i<25; ++i) {
        for(int j=0; j<25; ++j) { wall_v[i][j] = false; wall_h[i][j] = false; }
    }
    for(int j=0; j<HASH_SIZE; ++j) { 
        hash_step[j] = -1; 
    }

    for (int i = 0; i < N; ++i) {
        string s; cin >> s;
        for (int j = 0; j < N - 1; ++j) wall_v[i][j] = (s[j] == '1');
    }
    for (int i = 0; i < N - 1; ++i) {
        string s; cin >> s;
        for (int j = 0; j < N; ++j) wall_h[i][j] = (s[j] == '1');
    }
    for (int k = 0; k < M; ++k) cin >> balls[k].r >> balls[k].c >> baskets[k].r >> baskets[k].c;

    precompute_distances(); 

    string fallback_ops = solve_greedy();

    history.reserve(5000000); 
    history.push_back({-1, ' '});

    uint64_t init_z = z_pos[0][0] ^ z_dir[0] ^ z_hold[45] ^ z_is_rec[0];

    vector<State> current_beam;
    current_beam.reserve(5500);
    current_beam.push_back({0, 0, 0, evaluate(0, 0, -1, 0, false, 0), 0, 0, 0, 0, 0, -1, false, ' ' , init_z});

    // 🎯 本命のビーム幅10000の全力一発勝負に戻します
    const int BEAM_WIDTH = 8000;
    
    int best_partial_history = 0;
    int best_partial_score = -1e9;
    int best_goal_history = -1;
    int best_goal_real_cost = -1;
    
    uint64_t goal_mask = (1ULL << M) - 1;
    
    vector<State> next_beam;
    next_beam.reserve(BEAM_WIDTH * 15);

    run_id++;

    for (int step = 1; step <= T; ++step) {
        if (duration_cast<duration<double>>(high_resolution_clock::now() - start_time).count() > TIME_LIMIT) break;

        next_beam.clear();
        int current_generation = run_id * 10000 + step;

        for (const State& s : current_beam) {
            // 🎯 案1：現在ターゲットマス（未回収ボール、または保持中のかご）にいるか判定
            bool at_target = false;
            if (s.holding == -1) {
                for (int k = 0; k < M; ++k) {
                    if (!(s.completed & (1ULL << k)) && s.r == balls[k].r && s.c == balls[k].c) {
                        at_target = true;
                        break;
                    }
                }
            } else {
                if (s.r == baskets[s.holding].r && s.c == baskets[s.holding].c) {
                    at_target = true;
                }
            }

            // 🎯 録画中かつターゲットに到達した場合、次のコマンドはM（録画終了）しか選べないように強制
            bool force_macro_end = s.is_rec && at_target;

            auto try_push = [&](int8_t nr, int8_t nc, int8_t ndir, int8_t nhold, uint64_t ncomp, 
                                uint64_t nreg, uint64_t nrec, bool nrec_f, char op, int nreal, uint64_t nz_hash) {
                if (nreal > T) return; 

                State ns = {ncomp, nreg, nrec, 0, s.history_idx, nreal, nr, nc, ndir, nhold, nrec_f, op, nz_hash};
                uint64_t h = nz_hash ^ (nreg * 0xbf58476d1ce4e5b9ULL) ^ (nrec * 0x94d049bb133111ebULL);
                uint32_t idx = h & (HASH_SIZE - 1);
                
                while (true) {
                    if (hash_step[idx] != current_generation) {
                        hash_key[idx] = h;
                        hash_step[idx] = current_generation; 
                        hash_real_cost[idx] = nreal;
                        hash_beam_idx[idx] = next_beam.size(); 
                        
                        ns.score = evaluate(nr, nc, nhold, ncomp, nrec_f, step);
                        next_beam.push_back(ns);
                        return;
                    }
                    if (hash_key[idx] == h) {
                        if (nreal < hash_real_cost[idx]) {
                            hash_real_cost[idx] = nreal;
                            ns.score = evaluate(nr, nc, nhold, ncomp, nrec_f, step);
                            next_beam[hash_beam_idx[idx]] = ns; 
                        }
                        return; 
                    }
                    idx = (idx + 1) & (HASH_SIZE - 1);
                }
            };

            // 1. 基本操作 (F, R, L, S) の遷移
            for (int i = 0; i < 4; ++i) {
                if (force_macro_end) continue;     // 🎯 案1: ターゲット到達時は移動・交換を禁止してMへ誘導
                if (s.is_rec && i == 3) continue;  // 🎯 案2: 録画中のS操作（交換）は汎用性を落とすため完全に禁止

                char op = OPS[i];
                if (op == 'L' && s.temp_op == 'R') continue;
                if (op == 'R' && s.temp_op == 'L') continue;

                int8_t nr = s.r, nc = s.c, ndir = s.dir, nhold = s.holding;
                uint64_t ncomp = s.completed;
                uint64_t nz_hash = s.z_hash;
                
                if (sim_base(nr, nc, ndir, nhold, ncomp, i, nz_hash)) {
                    uint64_t nrec = s.rec_macro;
                    if (s.is_rec) {
                        int cur_len = (nrec & 0x1F) + 1;
                        if (cur_len > MAX_MACRO_LEN) continue; 
                        nrec = append_op(nrec, i);
                    }
                    try_push(nr, nc, ndir, nhold, ncomp, s.reg_macro, nrec, s.is_rec, op, s.real_cost + 1, nz_hash);
                }
            }

            // 2. M (マクロ開始/終了) の遷移
            if (s.temp_op != 'M') { 
                if (!s.is_rec && (T - s.real_cost < MAX_MACRO_LEN)) {
                    // 何もしない（残りコスト不足時の録画開始禁止）
                } else {
                    bool nrec_f = !s.is_rec;
                    uint64_t nreg = s.is_rec ? s.rec_macro : s.reg_macro;
                    uint64_t nrec = s.is_rec ? 0 : 0;
                    uint64_t nz_hash = s.z_hash ^ z_is_rec[s.is_rec] ^ z_is_rec[nrec_f];
                    try_push(s.r, s.c, s.dir, s.holding, s.completed, nreg, nrec, nrec_f, 'M', s.real_cost, nz_hash);
                }
            }

            // 3. P (マクロ再生) の遷移
            if (!force_macro_end && (s.reg_macro & 0x1F) > 0) { 
                int8_t nr = s.r, nc = s.c, ndir = s.dir, nhold = s.holding;
                uint64_t ncomp = s.completed;
                uint64_t nz_hash = s.z_hash;
                bool ok = true;
                
                uint64_t m_len = s.reg_macro & 0x1F; 
                uint64_t m_seq = s.reg_macro >> 5;   
                
                for (int k = 0; k < m_len; ++k) {
                    int op_idx = (m_seq >> (k * 2)) & 3;
                    if (!sim_base(nr, nc, ndir, nhold, ncomp, op_idx, nz_hash)) { ok = false; break; }
                }
                
                if (ok) {
                    uint64_t nrec = s.rec_macro;
                    if (s.is_rec) {
                        int new_len = (nrec & 0x1F) + m_len;
                        if (new_len <= MAX_MACRO_LEN) { 
                            nrec = append_macro(nrec, s.reg_macro);
                        } else {
                            ok = false; 
                        }
                    }
                    if (ok) try_push(nr, nc, ndir, nhold, ncomp, s.reg_macro, nrec, s.is_rec, 'P', s.real_cost + m_len, nz_hash);
                }
            }
        }

        if (next_beam.empty()) break;

        if (next_beam.size() > BEAM_WIDTH) {
            nth_element(next_beam.begin(), next_beam.begin() + BEAM_WIDTH, next_beam.end(), 
                        [](const State& a, const State& b) { return a.score > b.score; });
            next_beam.resize(BEAM_WIDTH);
        }

        current_beam.clear();
        for (State& ns : next_beam) {
            history.push_back({ns.history_idx, ns.temp_op});
            ns.history_idx = history.size() - 1;
            
            if (ns.completed == goal_mask && ns.real_cost <= T) {
                best_goal_history = ns.history_idx;
                best_goal_real_cost = ns.real_cost;
                goto END_SEARCH; 
            }

            if (ns.score > best_partial_score) {
                best_partial_score = ns.score;
                best_partial_history = ns.history_idx;
            }
            
            current_beam.push_back(ns);
        }
    }

END_SEARCH:
    if (best_goal_history != -1) {
        int curr = best_goal_history;
        string final_ops = "";
        while (curr > 0) {
            final_ops += history[curr].op;
            curr = history[curr].parent_idx;
        }
        reverse(final_ops.begin(), final_ops.end());
        for (char c : final_ops) cout << c << "\n";
        cerr << "[DEBUG] Beam Search Reached Goal! cmd_len: " << final_ops.length() 
             << " / expanded_cost: " << best_goal_real_cost << "\n";
    } else {
        for (char c : fallback_ops) cout << c << "\n";
        cerr << "[DEBUG] Beam Search Failed. Using Fallback Greedy! cmd_len: " << fallback_ops.length() << "\n";
    }
    
    return 0;
}