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

const int MAX_MACRO_LEN = 20;   
const char OPS[] = {'F', 'R', 'L', 'S'};

int N, M, T;

bool wall_v[25][25];
bool wall_h[25][25];
int dist_ball[45][25][25];
int dist_basket[45][25][25];

struct Pos { int r, c; };
Pos balls[45], baskets[45];

// 🚀 ビットエンコードマクロ機構 (長さ20対応, 40ビット使用)
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
    int8_t r, c, dir, holding;
    bool is_rec;
    char temp_op; 
};

// 世代管理ハッシュ配列
const int HASH_SIZE = 1 << 22; 
uint64_t hash_key[HASH_SIZE];
int hash_step[HASH_SIZE];      

inline uint64_t get_hash(const State& s) {
    uint64_t h = 14695981039346656037ULL;
    h ^= s.completed; h *= 1099511628211ULL;
    h ^= s.r;         h *= 1099511628211ULL;
    h ^= s.c;         h *= 1099511628211ULL;
    h ^= s.dir;       h *= 1099511628211ULL;
    h ^= s.holding;   h *= 1099511628211ULL;
    h ^= s.reg_macro; h *= 1099511628211ULL;
    h ^= s.rec_macro; h *= 1099511628211ULL;
    h ^= s.is_rec;    h *= 1099511628211ULL;
    return h;
}

inline bool insert_local_hash(uint64_t h, int current_step) {
    uint32_t idx = h & (HASH_SIZE - 1);
    while (true) {
        if (hash_step[idx] != current_step) {
            hash_key[idx] = h;
            hash_step[idx] = current_step; 
            return true;
        }
        if (hash_key[idx] == h) return false; 
        idx = (idx + 1) & (HASH_SIZE - 1);
    }
}

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

// 🎯【極限の純粋化】人工的なボーナスをすべて撤廃。純粋にステップ数と距離だけで勝負
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
    // 録画中の微小なペナルティのみ残す（ダラダラ録画し続けるのを防ぐため）
    if (is_rec) score -= 2; 
    
    // 最大の評価軸は「いかに少ない手数(step)で距離を詰めたか」のみ
    return score - step; 
}

inline bool sim_base(int8_t &r, int8_t &c, int8_t &dir, int8_t &holding, uint64_t &completed, int op_idx) {
    if (op_idx == 0) {
        int nr = r + dr[dir], nc = c + dc[dir];
        bool hit = false;
        if (nr < 0 || nr >= N || nc < 0 || nc >= N) hit = true;
        else if (dir == 0 && wall_v[r][c]) hit = true;
        else if (dir == 1 && wall_h[r][c]) hit = true;
        else if (dir == 2 && wall_v[r][c - 1]) hit = true;
        else if (dir == 3 && wall_h[r - 1][c]) hit = true;
        if (!hit) { r = nr; c = nc; }
        return true;
    } else if (op_idx == 1) {
        dir = (dir + 1) % 4; return true;
    } else if (op_idx == 2) {
        dir = (dir + 3) % 4; return true;
    } else {
        if (holding == -1) {
            for (int k = 0; k < M; ++k) {
                if (!(completed & (1ULL << k)) && r == balls[k].r && c == balls[k].c) {
                    holding = k; return true;
                }
                if ((completed & (1ULL << k)) && r == baskets[k].r && c == baskets[k].c) {
                    return false; 
                }
            }
            return true; 
        } else {
            if (r == baskets[holding].r && c == baskets[holding].c) {
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
    double TIME_LIMIT = 1.85; 

    for(int i=0; i<25; ++i) {
        for(int j=0; j<25; ++j) { wall_v[i][j] = false; wall_h[i][j] = false; }
        for(int j=0; j<HASH_SIZE; ++j) hash_step[j] = -1; 
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

    history.reserve(5000000); 
    history.push_back({-1, ' '});

    vector<State> current_beam;
    current_beam.reserve(5500);
    current_beam.push_back({0, 0, 0, evaluate(0, 0, -1, 0, false, 0), 0, 0, 0, 0, -1, false, ' '});

    // 🎯 ビーム幅を5000に設定
    const int BEAM_WIDTH = 5000;
    int best_goal_history = -1;
    int best_score = -1e9;
    uint64_t goal_mask = (1ULL << M) - 1;
    
    vector<State> next_beam;
    next_beam.reserve(BEAM_WIDTH * 15);

    for (int step = 1; step <= T; ++step) {
        if (duration_cast<duration<double>>(high_resolution_clock::now() - start_time).count() > TIME_LIMIT) break;

        next_beam.clear();

        for (const State& s : current_beam) {
            auto try_push = [&](int8_t nr, int8_t nc, int8_t ndir, int8_t nhold, uint64_t ncomp, 
                                uint64_t nreg, uint64_t nrec, bool nrec_f, char op) {
                State ns = {ncomp, nreg, nrec, 0, s.history_idx, nr, nc, ndir, nhold, nrec_f, op};
                uint64_t h = get_hash(ns);
                if (!insert_local_hash(h, step)) return; 

                // 評価関数にはボーナス変数は渡さない
                ns.score = evaluate(nr, nc, nhold, ncomp, nrec_f, step);
                next_beam.push_back(ns);
            };

            for (int i = 0; i < 4; ++i) {
                char op = OPS[i];
                if (op == 'L' && s.temp_op == 'R') continue;
                if (op == 'R' && s.temp_op == 'L') continue;

                int8_t nr = s.r, nc = s.c, ndir = s.dir, nhold = s.holding;
                uint64_t ncomp = s.completed;
                if (sim_base(nr, nc, ndir, nhold, ncomp, i)) {
                    uint64_t nrec = s.rec_macro;
                    if (s.is_rec) {
                        int cur_len = (nrec & 0x1F) + 1;
                        // 🎯 唯一のキャップは「絶対的なシステム上限(20文字)」のみ
                        if (cur_len > MAX_MACRO_LEN) continue; 
                        nrec = append_op(nrec, i);
                    }
                    try_push(nr, nc, ndir, nhold, ncomp, s.reg_macro, nrec, s.is_rec, op);
                }
            }

            if (s.temp_op != 'M') { 
                bool nrec_f = !s.is_rec;
                uint64_t nreg = s.is_rec ? s.rec_macro : s.reg_macro;
                uint64_t nrec = s.is_rec ? 0 : 0;
                try_push(s.r, s.c, s.dir, s.holding, s.completed, nreg, nrec, nrec_f, 'M');
            }

            if ((s.reg_macro & 0x1F) > 0) { 
                int8_t nr = s.r, nc = s.c, ndir = s.dir, nhold = s.holding;
                uint64_t ncomp = s.completed;
                bool ok = true;
                
                uint64_t m_len = s.reg_macro & 0x1F; 
                uint64_t m_seq = s.reg_macro >> 5;   
                
                for (int k = 0; k < m_len; ++k) {
                    int op_idx = (m_seq >> (k * 2)) & 3;
                    if (!sim_base(nr, nc, ndir, nhold, ncomp, op_idx)) { ok = false; break; }
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
                    if (ok) try_push(nr, nc, ndir, nhold, ncomp, s.reg_macro, nrec, s.is_rec, 'P');
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
            
            if (ns.score > best_score) {
                best_score = ns.score;
                best_goal_history = ns.history_idx;
            }
            if (ns.completed == goal_mask) {
                best_goal_history = ns.history_idx;
                goto END_SEARCH; 
            }
            current_beam.push_back(ns);
        }
    }

END_SEARCH:
    string final_ops = "";
    int curr = best_goal_history;
    while (curr > 0) {
        final_ops += history[curr].op;
        curr = history[curr].parent_idx;
    }
    reverse(final_ops.begin(), final_ops.end());

    if (final_ops.length() > T) final_ops = final_ops.substr(0, T);
    for (char c : final_ops) cout << c << "\n";
    cerr << "[DEBUG] len: " << final_ops.length() << " / score: " << best_score << "\n";
    return 0;
}