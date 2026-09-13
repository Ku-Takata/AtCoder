#include <iostream>
#include <vector>
#include <queue>
#include <cmath>
#include <algorithm>
#include <cstring>
#include <ctime>
#include <iomanip>
#include <tuple>
using namespace std;

const int MAXN = 50;
const int dr[] = {-1, 1, 0, 0};
const int dc[] = {0, 0, -1, 1};

int N, M;
double R;
char grid[MAXN][MAXN];
int own[MAXN][MAXN]; // -1 = free

struct Group {
    int p;
    long long s, t, v;
    double c_min;
    vector<pair<int,int>> cells;
    bool active;
};

Group grp[1000];
clock_t start_time;

int bps[MAXN+1][MAXN+1];
int ponds_bps[MAXN+1][MAXN+1];

void compute_bps() {
    memset(bps, 0, sizeof(bps));
    for (int i = 1; i <= N; i++)
        for (int j = 1; j <= N; j++)
            bps[i][j] = (grid[i-1][j-1] != '.' || own[i-1][j-1] != -1)
                      + bps[i-1][j] + bps[i][j-1] - bps[i-1][j-1];
}

void init_ponds() {
    memset(ponds_bps, 0, sizeof(ponds_bps));
    for (int i = 1; i <= N; i++)
        for (int j = 1; j <= N; j++)
            ponds_bps[i][j] = (grid[i-1][j-1] == '#')
                            + ponds_bps[i-1][j] + ponds_bps[i][j-1] - ponds_bps[i-1][j-1];
}

int count_blocked(int r, int c, int h, int w) {
    return bps[r+h][c+w] - bps[r][c+w] - bps[r+h][c] + bps[r][c];
}

int count_free(int r, int c, int h, int w) {
    return h * w - count_blocked(r, c, h, w);
}

int count_ponds(int r, int c, int h, int w) {
    return ponds_bps[r+h][c+w] - ponds_bps[r][c+w] - ponds_bps[r+h][c] + ponds_bps[r][c];
}

double compactness(const vector<pair<int,int>>& cells) {
    int p = cells.size();
    if (p == 0) return 0;
    bool m[MAXN][MAXN];
    memset(m, 0, sizeof(m));
    for (const auto& [x, y] : cells) m[x][y] = true;
    int L = 0;
    for (const auto& [x, y] : cells)
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx < 0 || nx >= N || ny < 0 || ny >= N || !m[nx][ny]) L++;
        }
    return 4.0 * sqrt((double)p) / L;
}





// 境界が壁や障害物、他グループと接している数（多いほど隙間を作らない良い配置）

double calc_time_bonus(const vector<pair<int,int>>& cells, long long dep_time) {
    double tb = 0.0;
    for (auto [x, y] : cells) {
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx >= 0 && nx < N && ny >= 0 && ny < N && own[nx][ny] != -1) {
                int neighbor_id = own[nx][ny];
                long long diff = abs(grp[neighbor_id].t - dep_time);
                tb += max(-1.0, 1.0 - (double)diff / 100.0);
            }
        }
    }
    return tb;
}

int calc_adjacency(const vector<pair<int,int>>& cells) {
    bool m[MAXN][MAXN];
    memset(m, 0, sizeof(m));
    for (const auto& [x, y] : cells) m[x][y] = true;
    
    int adj = 0;
    for (const auto& [x, y] : cells) {
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx < 0 || nx >= N || ny < 0 || ny >= N) {
                adj++;
            } else if (!m[nx][ny]) {
                if (grid[nx][ny] == '#' || own[nx][ny] >= 0) {
                    adj++;
                }
            }
        }
    }
    return adj;
}

void clear_cells(const vector<pair<int,int>>& cells) {
    for (const auto& [x, y] : cells) own[x][y] = -1;
}
void set_cells(const vector<pair<int,int>>& cells, int id) {
    for (const auto& [x, y] : cells) own[x][y] = id;
}





double global_avg_rem_time = 1000.0;
double W_ADJ = 0.003;
double W_TIME = 0.002;

double global_avg_density = 0.0;
double global_total_density = 0.0;
int global_density_count = 0;
long long global_total_rem_time = 0;
int global_groups_seen = 0;

double get_tie_breaker(const vector<pair<int,int>>& cells, long long rem_time, long long dep_time = 0) {
    if (cells.empty()) return 0.0;
    double cx = 0, cy = 0;
    bool touch_top = false, touch_bottom = false, touch_left = false, touch_right = false;
    for (auto [x, y] : cells) {
        cx += x; cy += y;
        if (x == 0) touch_top = true;
        if (x == N - 1) touch_bottom = true;
        if (y == 0) touch_left = true;
        if (y == N - 1) touch_right = true;
    }
    cx /= cells.size(); cy /= cells.size();
    
    double dx = cx - (N - 1) / 2.0;
    double dy = cy - (N - 1) / 2.0;
    double dist = sqrt(dx*dx + dy*dy);
    double max_dist = N / 1.414;
    double norm_dist = dist / max_dist;
    
    double time_factor = (double)rem_time / max(1.0, global_avg_rem_time) - 1.0;
    double score = time_factor * norm_dist;
    int corner_anchor = (touch_top && touch_left) + (touch_top && touch_right) + (touch_bottom && touch_left) + (touch_bottom && touch_right);
    if (corner_anchor > 0) score += 0.20 * corner_anchor;
    
    if (dep_time > 80000) {
        score += 0.08 * (double)(dep_time - 80000) / 20000.0 * norm_dist;
    }
    return score;
}

// 配置が空きスペースに面する辺の数（少ないほど空きスペースを分断しない）
int calc_exposed_free(const vector<pair<int,int>>& cells) {
    bool m[MAXN][MAXN];
    memset(m, 0, sizeof(m));
    for (const auto& [x, y] : cells) m[x][y] = true;
    
    int exposed = 0;
    for (const auto& [x, y] : cells) {
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx >= 0 && nx < N && ny >= 0 && ny < N && !m[nx][ny]) {
                if (grid[nx][ny] == '.' && own[nx][ny] == -1) {
                    exposed++;
                }
            }
        }
    }
    return exposed;
}

double calc_pipeline_bonus(long long dep_time) {
    int count_near = 0;
    for (int j = 0; j < 1000; j++) {
        if (grp[j].active && abs(grp[j].t - dep_time) < 1000) {
            count_near++;
        }
    }
    return max(0.0, 1.0 - (double)count_near / 4.0);
}

struct PlaceResult {
    vector<pair<int,int>> cells;
    double compact;
    int adj;
    double tie_breaker;
    double time_bonus;
    int exposed_free; // 空きスペースに面する辺の数
    int max_free_rect; // 配置後の残存最大空き長方形
    double pipe_bonus; // 退去時刻パイプライン分散ボーナス
};

double W_EXPOSE = 0.001; // 空きスペース露出ペナルティの重み
double W_ZONE = 0.01; // 滞在時間によるゾーン配置重み（長期＝壁際/端、短期＝中央）
double W_CAPACITY = 0.001; // 残存最大空き長方形（Future Capacity Index）の重み
double W_PIPE = 0.002; // 退去時刻パイプライン分散ボーナスの重み

// 50x50グリッド内での最大空き長方形（正方形優遇）の評価スコアをO(N^2)ヒストグラムDPで高速計算
int calc_max_free_rect() {
    int h[MAXN];
    memset(h, 0, sizeof(h));
    int max_score = 0;
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            if (grid[i][j] == '.' && own[i][j] == -1) h[j]++;
            else h[j] = 0;
        }
        vector<int> st;
        for (int j = 0; j <= N; j++) {
            int val = (j < N ? h[j] : 0);
            while (!st.empty() && h[st.back()] >= val) {
                int height = h[st.back()];
                st.pop_back();
                int width = st.empty() ? j : (j - st.back() - 1);
                int score = height * width - 10 * abs(height - width);
                max_score = max(max_score, score);
            }
            st.push_back(j);
        }
    }
    return max_score;
}

int eval_cap(const vector<pair<int,int>>& cells) {
    for (auto [x, y] : cells) own[x][y] = -2;
    int cap = calc_max_free_rect();
    for (auto [x, y] : cells) own[x][y] = -1;
    return cap;
}

// より良い配置か判定
bool is_better(const PlaceResult& a, const PlaceResult& b) {
    if (a.cells.empty()) return false;
    if (b.cells.empty()) return true;
    
    double score_a = a.compact + W_ADJ * a.adj + W_TIME * a.time_bonus - W_EXPOSE * a.exposed_free + W_ZONE * a.tie_breaker + W_CAPACITY * a.max_free_rect + W_PIPE * a.pipe_bonus;
    double score_b = b.compact + W_ADJ * b.adj + W_TIME * b.time_bonus - W_EXPOSE * b.exposed_free + W_ZONE * b.tie_breaker + W_CAPACITY * b.max_free_rect + W_PIPE * b.pipe_bonus;
    
    if (abs(score_a - score_b) < 1e-6) {
        if (a.adj != b.adj) return a.adj > b.adj;
        return a.tie_breaker > b.tie_breaker;
    }
    return score_a > score_b;
}

// === Box内ガイドBFS ===
vector<pair<int,int>> build_box_bfs(int r, int c, int bh, int bw, int p) {
    double cx = r + bh / 2.0;
    double cy = c + bw / 2.0;
    
    int sx = -1, sy = -1;
    double best_d = 1e18;
    for (int i = r; i < r + bh; i++)
        for (int j = c; j < c + bw; j++)
            if (grid[i][j] == '.' && own[i][j] == -1) {
                double d = (i - cx) * (i - cx) + (j - cy) * (j - cy);
                if (d < best_d) { best_d = d; sx = i; sy = j; }
            }
    
    if (sx < 0) return {};
    
    bool vis[MAXN][MAXN];
    memset(vis, 0, sizeof(vis));
    vis[sx][sy] = true;
    
    using E = pair<double, pair<int,int>>;
    priority_queue<E, vector<E>, greater<E>> pq;
    pq.push({0, {sx, sy}});
    
    vector<pair<int,int>> reg;
    while (!pq.empty() && (int)reg.size() < p) {
        auto [dist, pos] = pq.top(); pq.pop();
        auto [x, y] = pos;
        reg.push_back({x, y});
        
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx >= r && nx < r + bh && ny >= c && ny < c + bw
                && !vis[nx][ny] && grid[nx][ny] == '.' && own[nx][ny] == -1) {
                vis[nx][ny] = true;
                double dd = (nx - cx) * (nx - cx) + (ny - cy) * (ny - cy);
                pq.push({dd, {nx, ny}});
            }
        }
    }
    
    return (int)reg.size() == p ? reg : vector<pair<int,int>>{};
}

// === ダイクストラ法（8近傍Euclidean Prim法）による最密ブロック生成 ===
vector<pair<int,int>> build_dijkstra_blob(int sx, int sy, int p) {
    if (grid[sx][sy] != '.' || own[sx][sy] != -1) return {};
    
    bool in_reg[MAXN][MAXN];
    memset(in_reg, 0, sizeof(in_reg));
    
    using E = pair<double, pair<int,int>>;
    priority_queue<E, vector<E>, greater<E>> pq;
    
    pq.push({0.0, {sx, sy}});
    
    double best_cost[MAXN][MAXN];
    for(int i=0; i<N; i++) for(int j=0; j<N; j++) best_cost[i][j] = 1e18;
    best_cost[sx][sy] = 0.0;
    
    vector<pair<int,int>> reg;
    while (!pq.empty() && (int)reg.size() < p) {
        auto [cost, pos] = pq.top(); pq.pop();
        auto [x, y] = pos;
        
        if (in_reg[x][y]) continue;
        
        in_reg[x][y] = true;
        reg.push_back({x, y});
        
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx >= 0 && nx < N && ny >= 0 && ny < N && !in_reg[nx][ny] 
                && grid[nx][ny] == '.' && own[nx][ny] == -1) {
                
                int shared = 0;
                for (int dd = 0; dd < 4; dd++) {
                    int nnx = nx + dr[dd], nny = ny + dc[dd];
                    if (nnx >= 0 && nnx < N && nny >= 0 && nny < N && in_reg[nnx][nny]) {
                        shared++;
                    }
                }
                
                int diag_shared = 0;
                int ddr[4] = {-1, -1, 1, 1}, ddc[4] = {-1, 1, -1, 1};
                for (int dd = 0; dd < 4; dd++) {
                    int nnx = nx + ddr[dd], nny = ny + ddc[dd];
                    if (nnx >= 0 && nnx < N && nny >= 0 && nny < N && in_reg[nnx][nny]) {
                        diag_shared++;
                    }
                }
                
                double dist = (nx - sx)*(nx - sx) + (ny - sy)*(ny - sy);
                double delta_L = 4.0 - 2.0 * shared - 0.20 * diag_shared;
                double new_cost = delta_L * 10000.0 + dist;
                
                if (new_cost < best_cost[nx][ny]) {
                    best_cost[nx][ny] = new_cost;
                    pq.push({new_cost, {nx, ny}});
                }
            }
        }
    }
    
    return (int)reg.size() == p ? reg : vector<pair<int,int>>{};
}



PlaceResult find_best_placement(int p, long long rem_time, long long dep_time, bool quick_mode = false) {
    compute_bps();
    PlaceResult best = {{}, -1.0, -1, -1e9, -1e9, 9999, 0, 0.0};
    
    auto get_rect_cells = [](int r, int c, int w, int h, int excess) {
        vector<pair<int,int>> cells;
        cells.reserve(w * h - excess);
        for (int i = r; i < r + h - (excess > 0 ? 1 : 0); i++)
            for (int j = c; j < c + w; j++)
                cells.push_back({i, j});
        if (excess > 0) {
            int last_w = w - excess;
            for (int j = c; j < c + last_w; j++)
                cells.push_back({r + h - 1, j});
        }
        return cells;
    };
    
    auto eval_approx_rect = [](int r, int c, int w, int h, int excess, long long dep_time, int& out_adj, int& out_ef, double& out_tbonus) {
        static bool mark[MAXN][MAXN];
        int last_w = w - excess;
        for (int i = r; i < r + h - (excess > 0 ? 1 : 0); i++)
            for (int j = c; j < c + w; j++)
                mark[i][j] = true;
        if (excess > 0) {
            for (int j = c; j < c + last_w; j++)
                mark[r + h - 1][j] = true;
        }
        
        int adj = 0, ef = 0;
        double tbonus = 0.0;
        for (int i = r; i < r + h - (excess > 0 ? 1 : 0); i++) {
            for (int j = c; j < c + w; j++) {
                for (int d = 0; d < 4; d++) {
                    int nx = i + dr[d], ny = j + dc[d];
                    if (nx < 0 || nx >= N || ny < 0 || ny >= N) adj++;
                    else if (!mark[nx][ny]) {
                        if (grid[nx][ny] == '#' || own[nx][ny] >= 0) {
                            adj++;
                            if (own[nx][ny] >= 0) {
                                int neighbor_id = own[nx][ny];
                                long long diff = abs(grp[neighbor_id].t - dep_time);
                                tbonus += max(-1.0, 1.0 - (double)diff / 100.0);
                            }
                        } else if (grid[nx][ny] == '.' && own[nx][ny] == -1) ef++;
                    }
                }
            }
        }
        if (excess > 0) {
            int i = r + h - 1;
            for (int j = c; j < c + last_w; j++) {
                for (int d = 0; d < 4; d++) {
                    int nx = i + dr[d], ny = j + dc[d];
                    if (nx < 0 || nx >= N || ny < 0 || ny >= N) adj++;
                    else if (!mark[nx][ny]) {
                        if (grid[nx][ny] == '#' || own[nx][ny] >= 0) {
                            adj++;
                            if (own[nx][ny] >= 0) {
                                int neighbor_id = own[nx][ny];
                                long long diff = abs(grp[neighbor_id].t - dep_time);
                                tbonus += max(-1.0, 1.0 - (double)diff / 100.0);
                            }
                        } else if (grid[nx][ny] == '.' && own[nx][ny] == -1) ef++;
                    }
                }
            }
        }
        
        for (int i = r; i < r + h - (excess > 0 ? 1 : 0); i++)
            for (int j = c; j < c + w; j++)
                mark[i][j] = false;
        if (excess > 0) {
            for (int j = c; j < c + last_w; j++)
                mark[r + h - 1][j] = false;
        }
        
        out_adj = adj;
        out_ef = ef;
        out_tbonus = tbonus;
    };

    struct FastRect {
        int r, c, w, h, excess;
        double comp;
        int adj;
        double tb;
        double tbonus;
        int ef;
        double pipe_bonus;
        double score;
    };
    
    double pipe_bonus = calc_pipeline_bonus(dep_time);
    
    auto get_rect_tb = [](int r, int c, int w, int h, long long rem_time, long long dep_time) {
        double cx = r + (h - 1) / 2.0;
        double cy = c + (w - 1) / 2.0;
        double dx = cx - (N - 1) / 2.0;
        double dy = cy - (N - 1) / 2.0;
        double dist = sqrt(dx*dx + dy*dy);
        double norm_dist = dist / (N / 1.414);
        double time_factor = (double)rem_time / max(1.0, global_avg_rem_time) - 1.0;
        time_factor = max(-1.0, min(1.0, time_factor));
        double score = time_factor * norm_dist;
        if (dep_time > 80000) {
            score += 0.08 * (double)(dep_time - 80000) / 20000.0 * norm_dist;
        }
        return score;
    };
    
    // === Dynamic Free Space Aspect Ratio ===
    int min_r = N, max_r = -1, min_c = N, max_c = -1;
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            if (grid[i][j] == '.' && own[i][j] == -1) {
                min_r = min(min_r, i); max_r = max(max_r, i);
                min_c = min(min_c, j); max_c = max(max_c, j);
            }
        }
    }
    double free_aspect = (max_c >= min_c && max_r >= min_r) ? (double)(max_c - min_c + 1) / (max_r - min_r + 1) : 1.0;
    bool is_elongated = (free_aspect > 1.4 || free_aspect < 0.7);

    // === Phase 1: 完全長方形 (w*h == p) ===
    {
        vector<pair<int,int>> dims;
        for (int w = 1; w * w <= p; w++) {
            if (p % w == 0) {
                int h = p / w;
                if (w <= N && h <= N) dims.push_back({w, h});
                if (w != h && h <= N && w <= N) dims.push_back({h, w});
            }
        }
        sort(dims.begin(), dims.end(), [free_aspect, is_elongated](auto& a, auto& b) {
            if (is_elongated) {
                double aspect_a = (double)a.first / a.second;
                double aspect_b = (double)b.first / b.second;
                return abs(log(aspect_a / free_aspect)) < abs(log(aspect_b / free_aspect));
            }
            return abs(a.first - a.second) < abs(b.first - b.second);
        });
        
        vector<FastRect> fast_rects;
        for (auto [w, h] : dims) {
            double comp = 2.0 * sqrt((double)p) / (w + h);
            for (int r = 0; r + h <= N; r++) {
                for (int c = 0; c + w <= N; c++) {
                    if (count_blocked(r, c, h, w) == 0) {
                        double tb = get_rect_tb(r, c, w, h, rem_time, dep_time);
                        int adj, ef;
                        double tbonus;
                        eval_approx_rect(r, c, w, h, 0, dep_time, adj, ef, tbonus);
                        double score = comp + W_ADJ * adj + W_TIME * tbonus - W_EXPOSE * ef + W_ZONE * tb + W_PIPE * pipe_bonus;
                        fast_rects.push_back({r, c, w, h, 0, comp, adj, tb, tbonus, ef, pipe_bonus, score});
                    }
                }
            }
        }
        
        if (!fast_rects.empty()) {
            sort(fast_rects.begin(), fast_rects.end(), [](const FastRect& a, const FastRect& b) {
                return a.score > b.score;
            });
            if (quick_mode) {
                const auto& fr = fast_rects[0];
                auto cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, 0);
                PlaceResult cand = {cells, fr.comp, fr.adj, fr.tb, fr.tbonus, fr.ef, 0, fr.pipe_bonus};
                if (is_better(cand, best)) best = cand;
            } else {
                if ((int)fast_rects.size() > 10) fast_rects.resize(10);
                
                for (const auto& fr : fast_rects) {
                    auto cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, 0);
                    int cap = eval_cap(cells);
                    PlaceResult cand = {cells, fr.comp, fr.adj, fr.tb, fr.tbonus, fr.ef, cap, fr.pipe_bonus};
                    if (is_better(cand, best)) {
                        best = cand;
                    }
                }
            }
        }
    }
    
    // === Phase 2: 近似長方形 ===
    {
        vector<tuple<int,int,int>> dims;
        int sq = max(2, (int)round(sqrt((double)p)));
        for (int w = 2; w <= min(N, sq + 5); w++) {
            int h = (p + w - 1) / w;
            if (h > N || h < 2) continue;
            int excess = w * h - p;
            if (excess > 0 && excess < w) {
                dims.push_back({w, h, excess});
            }
        }
        sort(dims.begin(), dims.end(), [free_aspect, is_elongated](auto& a, auto& b) {
            if (is_elongated) {
                double aspect_a = (double)get<0>(a) / get<1>(a);
                double aspect_b = (double)get<0>(b) / get<1>(b);
                return abs(log(aspect_a / free_aspect)) < abs(log(aspect_b / free_aspect));
            }
            return abs(get<0>(a) - get<1>(a)) < abs(get<0>(b) - get<1>(b));
        });
        if ((int)dims.size() > 15) dims.resize(15);
        
        vector<FastRect> p2_fast;
        for (auto [w, h, excess] : dims) {
            int last_w = w - excess;
            for (int r = 0; r + h <= N; r++) {
                for (int c = 0; c + w <= N; c++) {
                    if (h >= 2 && count_blocked(r, c, h-1, w) == 0 && count_blocked(r+h-1, c, 1, last_w) == 0) {
                        double comp = 4.0 * sqrt((double)p) / (2 * (w + h) + (excess > 0 ? 2 : 0));
                        double tb = get_rect_tb(r, c, w, h, rem_time, dep_time);
                        int adj, ef;
                        double tbonus;
                        eval_approx_rect(r, c, w, h, excess, dep_time, adj, ef, tbonus);
                        double score = comp + W_ADJ * adj + W_TIME * tbonus - W_EXPOSE * ef + W_ZONE * tb + W_PIPE * pipe_bonus;
                        p2_fast.push_back({r, c, w, h, excess, comp, adj, tb, tbonus, ef, pipe_bonus, score});
                    }
                }
            }
        }
        
        if (!p2_fast.empty()) {
            sort(p2_fast.begin(), p2_fast.end(), [](const FastRect& a, const FastRect& b) {
                return a.score > b.score;
            });
            if (quick_mode) {
                const auto& fr = p2_fast[0];
                auto cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, fr.excess);
                double comp = compactness(cells);
                PlaceResult cand = {cells, comp, fr.adj, fr.tb, fr.tbonus, fr.ef, 0, fr.pipe_bonus};
                if (is_better(cand, best)) best = cand;
            } else {
                if ((int)p2_fast.size() > 10) p2_fast.resize(10);
                
                for (const auto& fr : p2_fast) {
                    auto cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, fr.excess);
                    double comp = compactness(cells);
                    int cap = eval_cap(cells);
                    PlaceResult cand = {cells, comp, fr.adj, fr.tb, fr.tbonus, fr.ef, cap, fr.pipe_bonus};
                    if (is_better(cand, best)) {
                        best = cand;
                    }
                }
            }
        }
    }
    
    double elapsed_now = (double)(clock() - start_time) / CLOCKS_PER_SEC;
    if (quick_mode || elapsed_now > 1.75 || best.compact > 0.85) return best;
    
    // === Phase 2.5: Box内BFS（障害物あり対応） ===
    {
        int sq = max(2, (int)round(sqrt((double)p)));
        
        vector<pair<int,int>> box_dims;
        for (int bw = sq; bw <= min(N, sq + 6); bw++) {
            for (int bh = max(2, (p + bw - 1) / bw); bh <= min(N, bw + 3); bh++) {
                if (bw * bh >= p) {
                    box_dims.push_back({bw, bh});
                    if (bw != bh) box_dims.push_back({bh, bw});
                }
            }
        }
        sort(box_dims.begin(), box_dims.end(), [](auto& a, auto& b) {
            int area_a = a.first * a.second, area_b = b.first * b.second;
            if (area_a != area_b) return area_a < area_b;
            return abs(a.first - a.second) < abs(b.first - b.second);
        });
        box_dims.erase(unique(box_dims.begin(), box_dims.end()), box_dims.end());
        if ((int)box_dims.size() > 20) box_dims.resize(20);
        
        vector<PlaceResult> p25_cands;
        for (auto [bw, bh] : box_dims) {
            if (bw > N || bh > N) continue;
            
            for (int r = 0; r + bh <= N; r++) {
                for (int c = 0; c + bw <= N; c++) {
                    int free = count_free(r, c, bh, bw);
                    if (free < p) continue;
                    
                    auto reg = build_box_bfs(r, c, bh, bw, p);
                    if (!reg.empty()) {
                        double comp = compactness(reg);
                        int adj = calc_adjacency(reg);
                        double tb = get_tie_breaker(reg, rem_time, dep_time);
                        int ef = calc_exposed_free(reg);
                        p25_cands.push_back({reg, comp, adj, tb, 0.0, ef, 0, pipe_bonus});
                    }
                }
            }
        }
        
        if (!p25_cands.empty()) {
            sort(p25_cands.begin(), p25_cands.end(), [](const PlaceResult& a, const PlaceResult& b) {
                double sa = a.compact + W_ADJ * a.adj + W_TIME * a.time_bonus - W_EXPOSE * a.exposed_free + W_ZONE * a.tie_breaker + W_PIPE * a.pipe_bonus;
                double sb = b.compact + W_ADJ * b.adj + W_TIME * b.time_bonus - W_EXPOSE * b.exposed_free + W_ZONE * b.tie_breaker + W_PIPE * b.pipe_bonus;
                return sa > sb;
            });
            if ((int)p25_cands.size() > 10) p25_cands.resize(10);
            
            for (auto& cand : p25_cands) {
                cand.max_free_rect = eval_cap(cand.cells);
                if (is_better(cand, best)) {
                    best = cand;
                }
            }
        }
    }
    
    elapsed_now = (double)(clock() - start_time) / CLOCKS_PER_SEC;
    if (elapsed_now > 1.75 || best.compact > 0.75) return best;
    
    // === Phase 3: ガイド付きBFS（連結成分ベース） ===
    {
        bool vis[MAXN][MAXN];
        memset(vis, 0, sizeof(vis));
        vector<PlaceResult> p3_cands;
        
        for (int i = 0; i < N; i++) {
            for (int j = 0; j < N; j++) {
                if (vis[i][j] || grid[i][j] != '.' || own[i][j] != -1) continue;
                
                queue<pair<int,int>> q;
                q.push({i, j});
                vis[i][j] = true;
                vector<pair<int,int>> comp;
                
                while (!q.empty()) {
                    auto [x, y] = q.front(); q.pop();
                    comp.push_back({x, y});
                    for (int d = 0; d < 4; d++) {
                        int nx = x + dr[d], ny = y + dc[d];
                        if (nx >= 0 && nx < N && ny >= 0 && ny < N && !vis[nx][ny]
                            && grid[nx][ny] == '.' && own[nx][ny] == -1) {
                            vis[nx][ny] = true;
                            q.push({nx, ny});
                        }
                    }
                }
                
                if ((int)comp.size() < p) continue;
                
                double cx = 0, cy = 0;
                for (const auto& [x, y] : comp) { cx += x; cy += y; }
                cx /= comp.size(); cy /= comp.size();
                
                sort(comp.begin(), comp.end(), [cx, cy](const auto& a, const auto& b) {
                    int adj_a = 0, adj_b = 0;
                    for (int d = 0; d < 4; d++) {
                        int nxa = a.first + dr[d], nya = a.second + dc[d];
                        if (nxa < 0 || nxa >= N || nya < 0 || nya >= N || grid[nxa][nya] == '#' || own[nxa][nya] >= 0) adj_a++;
                        int nxb = b.first + dr[d], nyb = b.second + dc[d];
                        if (nxb < 0 || nxb >= N || nyb < 0 || nyb >= N || grid[nxb][nyb] == '#' || own[nxb][nyb] >= 0) adj_b++;
                    }
                    double da = (a.first-cx)*(a.first-cx) + (a.second-cy)*(a.second-cy) - adj_a * 40.0;
                    double db = (b.first-cx)*(b.first-cx) + (b.second-cy)*(b.second-cy) - adj_b * 40.0;
                    return da < db;
                });
                
                double elapsed_now = (double)(clock() - start_time) / CLOCKS_PER_SEC;
                int max_t = (elapsed_now < 0.80) ? 96 : ((elapsed_now < 1.40) ? 48 : 24);
                int trials = min((int)comp.size(), max_t);
                for (int t = 0; t < trials; t++) {
                    auto reg = build_dijkstra_blob(comp[t].first, comp[t].second, p);
                    if (!reg.empty()) {
                        double comp_val = compactness(reg);
                        int adj = calc_adjacency(reg);
                        double tb = get_tie_breaker(reg, rem_time, dep_time);
                        double tbonus = calc_time_bonus(reg, dep_time);
                        int ef = calc_exposed_free(reg);
                        p3_cands.push_back({reg, comp_val, adj, tb, tbonus, ef, 0, pipe_bonus});
                    }
                }
            }
        }
        
        if (!p3_cands.empty()) {
            sort(p3_cands.begin(), p3_cands.end(), [](const PlaceResult& a, const PlaceResult& b) {
                double sa = a.compact + W_ADJ * a.adj + W_TIME * a.time_bonus - W_EXPOSE * a.exposed_free + W_ZONE * a.tie_breaker + W_PIPE * a.pipe_bonus;
                double sb = b.compact + W_ADJ * b.adj + W_TIME * b.time_bonus - W_EXPOSE * b.exposed_free + W_ZONE * b.tie_breaker + W_PIPE * b.pipe_bonus;
                return sa > sb;
            });
            if ((int)p3_cands.size() > 10) p3_cands.resize(10);
            
            for (auto& cand : p3_cands) {
                cand.max_free_rect = eval_cap(cand.cells);
                if (is_better(cand, best)) {
                    best = cand;
                }
            }
        }
    }
    
    return best;
}

// 診断情報
struct Diag {
    int accepted = 0, rejected = 0;
    double total_compact = 0;
    int phase1_count = 0, phase2_count = 0, phase25_count = 0, phase3_count = 0;
    long long total_fee = 0, total_move_cost = 0;
    int move_count = 0;
    int bucket_n[4] = {};
    double bucket_compact_sum[4] = {};
    long long bucket_fee[4] = {};
    int bucket_reject[4] = {};
} diag;

int size_bucket(int p) {
    if (p <= 30) return 0;
    if (p <= 70) return 1;
    if (p <= 110) return 2;
    return 3;
}

struct MultiMovePlan {
    vector<pair<int,int>> icells;
    long long net_gain;
    vector<int> intersecting;
    vector<pair<int, vector<pair<int,int>>>> move_plans;
};

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);
    
    start_time = clock();
    
    if (!(cin >> N >> M >> R)) return 0;
    
    int total_grass = 0;
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            cin >> grid[i][j];
            if (grid[i][j] == '.') total_grass++;
        }
    }
    
    init_ponds();
    memset(own, -1, sizeof(own));
    
    for (int turn = 0; turn < M; turn++) {
        int id;
        if (!(cin >> id)) break;
        cin >> grp[turn].s >> grp[turn].t >> grp[turn].p >> grp[turn].v;
        grp[turn].c_min = 0.0;
        grp[turn].active = false;
        
        long long now = grp[turn].s;
        long long rem_time = grp[turn].t - now;
        int p = grp[turn].p;
        long long v = grp[turn].v;
        
        global_total_rem_time += rem_time;
        global_groups_seen++;
        global_avg_rem_time = (double)global_total_rem_time / global_groups_seen;
        
        double density = (double)v / ((double)p * rem_time);
        global_total_density += density;
        global_density_count++;
        global_avg_density = global_total_density / global_density_count;
        
        int cur_free = 0;
        for (int i = 0; i < N; i++)
            for (int j = 0; j < N; j++)
                if (grid[i][j] == '.' && own[i][j] == -1) cur_free++;
        double free_rate = (double)cur_free / total_grass;
        double p_factor = 0.90 + 0.20 * ((double)p / 150.0);
        double current_thresh = (turn >= M - 40) ? 0.0 : (0.25 + 0.60 * (1.0 - free_rate)) * p_factor;
        if (global_density_count > 20 && density < global_avg_density * current_thresh) {
            cout << "0\nNo\n" << flush;
            diag.rejected++;
            diag.bucket_reject[size_bucket(p)]++;
            continue;
        }
        
        for (int j = 0; j < turn; j++) {
            if (grp[j].active && grp[j].t < now) {
                long long fee = (long long)floor(grp[j].v * grp[j].c_min + 0.5);
                diag.total_fee += fee;
                diag.bucket_fee[size_bucket(grp[j].p)] += fee;
                clear_cells(grp[j].cells);
                grp[j].cells.clear();
                grp[j].active = false;
            }
        }
        
        auto result = find_best_placement(p, rem_time, grp[turn].t);
        bool try_multimove = result.cells.empty() 
                           || (R <= 0.03 && result.compact < 0.80 && v > 20000000LL) 
                           || (result.compact < 0.72 && v > 50000000LL);
        
        if (!try_multimove) {
            cout << "0\nYes\n";
            for (const auto& [x, y] : result.cells)
                cout << x << " " << y << "\n";
            set_cells(result.cells, turn);
            grp[turn].cells = result.cells;
            grp[turn].c_min = result.compact;
            grp[turn].active = true;
            
            diag.accepted++;
            diag.total_compact += result.compact;
            int b = size_bucket(p);
            diag.bucket_n[b]++;
            diag.bucket_compact_sum[b] += result.compact;
            
            if (result.compact > 0.95) diag.phase1_count++;
            else if (result.compact > 0.85) diag.phase2_count++;
            else if (result.compact > 0.75) diag.phase25_count++;
            else diag.phase3_count++;
        } else {
            MultiMovePlan best_mm;
            best_mm.net_gain = !result.cells.empty() ? (long long)floor(v * result.compact + 0.5) : 0;
            
            double elapsed = (double)(clock() - start_time) / CLOCKS_PER_SEC;
            if (elapsed < 1.60) {
                vector<pair<int,int>> box_shapes;
                for (int w = 1; w <= N; w++) {
                    int h = (p + w - 1) / w;
                    if (h >= 1 && h <= N) {
                        int excess = h * w - p;
                        if (excess >= 0 && excess < w && (double)max(h, w) / min(h, w) <= 2.2) box_shapes.push_back({h, w});
                    }
                }
                
                sort(box_shapes.begin(), box_shapes.end(), [](auto& a, auto& b) {
                    return abs(a.first - a.second) < abs(b.first - b.second);
                });
                
                for (auto [h, w] : box_shapes) {
                    int excess = h * w - p;
                    vector<pair<int,int>> icells;
                    for(int i=0; i<h-1; i++) for(int j=0; j<w; j++) icells.push_back({i, j});
                    for(int j=0; j<w-excess; j++) icells.push_back({h-1, j});
                    double box_comp = compactness(icells);
                    long long base_expected_fee = floor(v * box_comp + 0.5);
                    
                    for (int r = 0; r + h <= N; r++) {
                        for (int c = 0; c + w <= N; c++) {
                            if ((double)(clock() - start_time) / CLOCKS_PER_SEC > 1.60) goto mm_end;
                            
                            int ponds = count_ponds(r, c, h - 1, w) + count_ponds(r + h - 1, c, 1, w - excess);
                            if (ponds > 0) continue;
                            
                            int occ = (count_blocked(r, c, h - 1, w) - count_ponds(r, c, h - 1, w))
                                    + (count_blocked(r + h - 1, c, 1, w - excess) - count_ponds(r + h - 1, c, 1, w - excess));
                            if (occ == 0 || occ > 60) continue; 
                            
                            vector<int> intersecting;
                            bool ok = true;
                            int max_inter = (R <= 0.005) ? 10 : (R <= 0.01 ? 8 : (R <= 0.02 ? 6 : (R <= 0.05 ? 4 : 3)));
                            for (int i = r; i < r + h && ok; i++) {
                                for (int j = c; j < c + w; j++) {
                                    if (i == r + h - 1 && j >= c + w - excess) continue;
                                    if (own[i][j] != -1) {
                                        int id = own[i][j];
                                        if (find(intersecting.begin(), intersecting.end(), id) == intersecting.end()) {
                                            intersecting.push_back(id);
                                            if ((int)intersecting.size() > max_inter) { ok = false; break; }
                                        }
                                    }
                                }
                            }
                            if (!ok || intersecting.empty()) continue;
                            
                            long long total_move_cost = 0;
                            for (int id : intersecting) total_move_cost += max(1LL, (long long)floor(grp[id].v * R + 0.5));
                            double cost_ratio = 0.35 + 0.30 * min(1.0, (double)v / 50000000.0);
                            if (total_move_cost > base_expected_fee * cost_ratio || base_expected_fee - total_move_cost <= best_mm.net_gain) continue;
                            
                            icells.clear();
                            for(int i=0; i<h-1; i++) for(int j=0; j<w; j++) icells.push_back({r+i, c+j});
                            for(int j=0; j<w-excess; j++) icells.push_back({r+h-1, c+j});
                            double actual_box_comp = compactness(icells);
                            long long expected_fee = floor(v * actual_box_comp + 0.5);
                            
                            for(int id : intersecting) clear_cells(grp[id].cells);
                            for(auto [cx, cy] : icells) own[cx][cy] = -2;
                            
                            vector<int> inter_sorted = intersecting;
                            sort(inter_sorted.begin(), inter_sorted.end(), [](int a, int b) {
                                return (grp[a].v * R) < (grp[b].v * R);
                            });
                            
                            long long actual_loss = 0;
                            bool possible = true;
                            vector<pair<int, vector<pair<int,int>>>> move_plans;
                            
                            for (int id : inter_sorted) {
                                if ((double)(clock() - start_time) / CLOCKS_PER_SEC > 1.60) {
                                    for(auto [cx, cy] : icells) own[cx][cy] = -1;
                                    for(int tid : intersecting) set_cells(grp[tid].cells, tid);
                                    compute_bps();
                                    goto mm_end;
                                }
                                auto res = find_best_placement(grp[id].p, grp[id].t - now, grp[id].t, true);
                                if (res.cells.empty()) { possible = false; break; }
                                set_cells(res.cells, id);
                                move_plans.push_back({id, res.cells});
                                actual_loss += (floor(grp[id].v * grp[id].c_min + 0.5) - floor(grp[id].v * min(grp[id].c_min, res.compact) + 0.5));
                            }
                            
                            for (auto& plan : move_plans) clear_cells(plan.second);
                            for(auto [cx, cy] : icells) own[cx][cy] = -1;
                            for(int id : intersecting) set_cells(grp[id].cells, id);
                            compute_bps();
                            
                            if (possible) {
                                long long net_gain = expected_fee - total_move_cost - actual_loss;
                                if (net_gain > best_mm.net_gain) {
                                    best_mm.net_gain = net_gain;
                                    best_mm.icells = icells;
                                    best_mm.intersecting = intersecting;
                                    best_mm.move_plans = move_plans;
                                }
                            }
                        }
                    }
                }
            }
            mm_end:;
            
            if (best_mm.net_gain > 0) {
                cout << best_mm.move_plans.size() << "\n";
                
                // 1. まず全て古いセルをクリアする
                for (const auto& plan : best_mm.move_plans) {
                    int id = plan.first;
                    clear_cells(grp[id].cells);
                }
                
                // 2. 移動の出力と新しいセルの設定
                for (const auto& plan : best_mm.move_plans) {
                    int id = plan.first;
                    cout << id << "\n";
                    for (const auto& [x, y] : plan.second) {
                        cout << x << " " << y << "\n";
                    }
                    long long mc = max(1LL, (long long)floor(grp[id].v * R + 0.5));
                    diag.total_move_cost += mc;
                    diag.move_count++;
                    
                    grp[id].cells = plan.second;
                    set_cells(plan.second, id);
                    grp[id].c_min = min(grp[id].c_min, compactness(plan.second));
                }
                
                cout << "Yes\n";
                for (const auto& [x, y] : best_mm.icells) {
                    cout << x << " " << y << "\n";
                }
                set_cells(best_mm.icells, turn);
                grp[turn].cells = best_mm.icells;
                grp[turn].c_min = compactness(best_mm.icells);
                grp[turn].active = true;
                
                diag.accepted++;
                diag.total_compact += grp[turn].c_min;
                int b = size_bucket(p);
                diag.bucket_n[b]++;
                diag.bucket_compact_sum[b] += grp[turn].c_min;
                diag.phase3_count++;
            } else if (!result.cells.empty()) {
                cout << "0\nYes\n";
                for (const auto& [x, y] : result.cells)
                    cout << x << " " << y << "\n";
                set_cells(result.cells, turn);
                grp[turn].cells = result.cells;
                grp[turn].c_min = result.compact;
                grp[turn].active = true;
                
                diag.accepted++;
                diag.total_compact += result.compact;
                int b = size_bucket(p);
                diag.bucket_n[b]++;
                diag.bucket_compact_sum[b] += result.compact;
                
                if (result.compact > 0.95) diag.phase1_count++;
                else if (result.compact > 0.85) diag.phase2_count++;
                else if (result.compact > 0.75) diag.phase25_count++;
                else diag.phase3_count++;
            } else {
                cout << "0\nNo\n";
                diag.rejected++;
                diag.bucket_reject[size_bucket(p)]++;
            }
        }
        cout << flush;
    }
    
    for (int j = 0; j < M; j++) {
        if (grp[j].active) {
            long long fee = (long long)floor(grp[j].v * grp[j].c_min + 0.5);
            diag.total_fee += fee;
            diag.bucket_fee[size_bucket(grp[j].p)] += fee;
        }
    }
    
    double elapsed = (double)(clock() - start_time) / CLOCKS_PER_SEC;
    const char* bnames[] = {"P=4-30", "P=31-70", "P=71-110", "P=111-150"};
    
    cerr << "\n==============================================" << endl;
    cerr << "   AHC069 High-Performance Diagnostics Report" << endl;
    cerr << "==============================================" << endl;
    cerr << "Execution Time: " << fixed << setprecision(3) << elapsed << " s / Limit: 2.000s" << endl;
    if (elapsed >= 1.50) {
        cerr << "[WARNING] High Execution Time (>1.5s)! Time Safety Guard Engaged." << endl;
    } else {
        cerr << "[STATUS] Execution Time SAFE (<1.5s)." << endl;
    }
    cerr << "Map Grass: " << total_grass << "/" << N*N 
         << " (" << fixed << setprecision(1) << 100.0*total_grass/(N*N) << "%)" << endl;
    cerr << "Acceptance: Accepted=" << diag.accepted << " / Rejected=" << diag.rejected 
         << " (Rate: " << fixed << setprecision(1) << 100.0*diag.accepted/max(1, diag.accepted+diag.rejected) << "%)" << endl;
    if (diag.accepted > 0)
        cerr << "Average Compactness: " << fixed << setprecision(4) << diag.total_compact / diag.accepted << endl;
    cerr << "Phase Distribution: Phase1=" << diag.phase1_count 
         << " | Phase2=" << diag.phase2_count 
         << " | Phase2.5=" << diag.phase25_count
         << " | Phase3=" << diag.phase3_count << endl;
    cerr << "Moves: " << diag.move_count << " (Cost: " << diag.total_move_cost << ")" << endl;
    cerr << "Net Fee: " << diag.total_fee << " - " << diag.total_move_cost << " = " << (diag.total_fee - diag.total_move_cost) << endl;
    cerr << "--- Per size bucket ---" << endl;
    for (int b = 0; b < 4; b++) {
        cerr << "  " << bnames[b] << ": acc=" << diag.bucket_n[b]
             << " rej=" << diag.bucket_reject[b];
        if (diag.bucket_n[b] > 0)
            cerr << " avgC=" << diag.bucket_compact_sum[b] / diag.bucket_n[b];
        cerr << " fee=" << diag.bucket_fee[b] << endl;
    }
    
    return 0;
}
