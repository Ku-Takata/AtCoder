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
    static bool m[MAXN][MAXN];
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

int calc_adjacency(const vector<pair<int,int>>& cells) {
    static bool m[MAXN][MAXN];
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

double calc_time_bonus(const vector<pair<int,int>>& cells, long long dep_time) {
    double bonus = 0.0;
    for (auto [x, y] : cells) {
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx >= 0 && nx < N && ny >= 0 && ny < N && own[nx][ny] >= 0) {
                int neighbor_id = own[nx][ny];
                long long t_neigh = grp[neighbor_id].t;
                long long diff = t_neigh - dep_time;
                
                if (diff == 0) {
                    // 同一ターン退去：巨大空間の統合形成のため強烈合体ボーナス
                    bonus += 2.5;
                } else if (diff > 0 && diff <= 120) {
                    // 連続退去カスケード：自グループの直後に退去し波状の空間拡張を生む
                    bonus += (1.5 - (double)diff / 120.0);
                } else {
                    // 長期留保ブロックペナルティ
                    bonus -= 0.5;
                }
            }
        }
    }
    return bonus;
}

double global_avg_rem_time = 1000.0;
double W_ADJ = 0.003;
double W_TIME = 0.005;
double REJ_THRESH = 0.70;

double global_avg_density = 0.0;
double global_total_density = 0.0;
int global_density_count = 0;
long long global_total_rem_time = 0;
int global_groups_seen = 0;

double get_tie_breaker(const vector<pair<int,int>>& cells, long long rem_time) {
    if (cells.empty()) return 0.0;
    double cx = 0, cy = 0;
    for (auto [x, y] : cells) { cx += x; cy += y; }
    cx /= cells.size(); cy /= cells.size();
    
    double dx = cx - (N - 1) / 2.0;
    double dy = cy - (N - 1) / 2.0;
    double dist = sqrt(dx*dx + dy*dy);
    double max_dist = N / 1.414;
    double norm_dist = dist / max_dist;
    
    double time_factor = (double)rem_time / max(1.0, global_avg_rem_time) - 1.0;
    time_factor = max(-1.0, min(1.0, time_factor));
    
    return time_factor * norm_dist;
}

int calc_exposed_free(const vector<pair<int,int>>& cells) {
    static bool m[MAXN][MAXN];
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

struct PlaceResult {
    vector<pair<int,int>> cells;
    double compact;
    int adj;
    double tie_breaker;
    double time_bonus;
    int exposed_free;
    int max_free_rect;
};

double W_EXPOSE = 0.001;
double W_ZONE = 0.01;
double W_CAPACITY = 0.001;

int calc_max_free_rect() {
    static int h[MAXN];
    memset(h, 0, sizeof(h));
    int max_area = 0;
    static int st[MAXN + 5];
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            if (grid[i][j] == '.' && own[i][j] == -1) h[j]++;
            else h[j] = 0;
        }
        int top = 0;
        for (int j = 0; j <= N; j++) {
            int val = (j < N ? h[j] : 0);
            while (top > 0 && h[st[top - 1]] >= val) {
                int height = h[st[--top]];
                int width = (top == 0) ? j : (j - st[top - 1] - 1);
                max_area = max(max_area, height * width);
            }
            st[top++] = j;
        }
    }
    return max_area;
}

int eval_cap(const vector<pair<int,int>>& cells) {
    for (auto [x, y] : cells) own[x][y] = -2;
    int cap = calc_max_free_rect();
    for (auto [x, y] : cells) own[x][y] = -1;
    return cap;
}

bool is_better(const PlaceResult& a, const PlaceResult& b) {
    if (a.cells.empty()) return false;
    if (b.cells.empty()) return true;
    
    double score_a = a.compact + W_ADJ * a.adj + W_TIME * a.time_bonus - W_EXPOSE * a.exposed_free + W_ZONE * a.tie_breaker + W_CAPACITY * a.max_free_rect;
    double score_b = b.compact + W_ADJ * b.adj + W_TIME * b.time_bonus - W_EXPOSE * b.exposed_free + W_ZONE * b.tie_breaker + W_CAPACITY * b.max_free_rect;
    
    if (abs(score_a - score_b) < 1e-6) {
        return a.compact > b.compact;
    }
    return score_a > score_b;
}

// === NEW FLEXIBLE SHAPE GENERATOR 1: Prim's Min-Perimeter Blob (Flexible Non-Rectangle Growth) ===
vector<pair<int,int>> build_min_perimeter_blob(int sx, int sy, int p) {
    if (grid[sx][sy] != '.' || own[sx][sy] != -1) return {};
    
    static bool in_blob[MAXN][MAXN];
    memset(in_blob, 0, sizeof(in_blob));
    
    vector<pair<int,int>> blob;
    blob.reserve(p);
    blob.push_back({sx, sy});
    in_blob[sx][sy] = true;
    
    auto calc_delta = [](int x, int y) {
        int neighbor_count = 0;
        int wall_count = 0;
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx < 0 || nx >= N || ny < 0 || ny >= N) wall_count++;
            else if (in_blob[nx][ny]) neighbor_count++;
            else if (grid[nx][ny] == '#' || own[nx][ny] >= 0) wall_count++;
        }
        // Incremental perimeter change if (x,y) is added
        // Base edge = 4, minus 2 for each existing blob neighbor, minus 0.5 for walls
        return 4 - 2 * neighbor_count - 0.5 * wall_count;
    };
    
    while ((int)blob.size() < p) {
        int best_x = -1, best_y = -1;
        double min_cost = 1e9;
        
        for (const auto& [x, y] : blob) {
            for (int d = 0; d < 4; d++) {
                int nx = x + dr[d], ny = y + dc[d];
                if (nx >= 0 && nx < N && ny >= 0 && ny < N && !in_blob[nx][ny]
                    && grid[nx][ny] == '.' && own[nx][ny] == -1) {
                    double cost = calc_delta(nx, ny);
                    if (cost < min_cost) {
                        min_cost = cost;
                        best_x = nx;
                        best_y = ny;
                    }
                }
            }
        }
        
        if (best_x == -1) break;
        in_blob[best_x][best_y] = true;
        blob.push_back({best_x, best_y});
    }
    
    return (int)blob.size() == p ? blob : vector<pair<int,int>>{};
}

// === PURE DYNAMIC SHAPE GENERATOR 1: Contour-Nook Priority Growth (No Templates) ===
vector<pair<int,int>> build_nook_priority_blob(int sx, int sy, int p) {
    if (grid[sx][sy] != '.' || own[sx][sy] != -1) return {};
    
    static bool in_blob[MAXN][MAXN];
    memset(in_blob, 0, sizeof(in_blob));
    
    vector<pair<int,int>> blob;
    blob.reserve(p);
    blob.push_back({sx, sy});
    in_blob[sx][sy] = true;
    
    auto calc_score = [](int x, int y) {
        int blob_neighbors = 0;
        int wall_neighbors = 0;
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx < 0 || nx >= N || ny < 0 || ny >= N) wall_neighbors++;
            else if (in_blob[nx][ny]) blob_neighbors++;
            else if (grid[nx][ny] == '#' || own[nx][ny] >= 0) wall_neighbors++;
        }
        // Score = Higher neighbor count (fill nooks first) + wall contact
        return 3.0 * blob_neighbors + 2.0 * wall_neighbors;
    };
    
    while ((int)blob.size() < p) {
        int best_x = -1, best_y = -1;
        double max_score = -1e9;
        
        for (const auto& [x, y] : blob) {
            for (int d = 0; d < 4; d++) {
                int nx = x + dr[d], ny = y + dc[d];
                if (nx >= 0 && nx < N && ny >= 0 && ny < N && !in_blob[nx][ny]
                    && grid[nx][ny] == '.' && own[nx][ny] == -1) {
                    double sc = calc_score(nx, ny);
                    if (sc > max_score) {
                        max_score = sc;
                        best_x = nx;
                        best_y = ny;
                    }
                }
            }
        }
        
        if (best_x == -1) break;
        in_blob[best_x][best_y] = true;
        blob.push_back({best_x, best_y});
    }
    
    return (int)blob.size() == p ? blob : vector<pair<int,int>>{};
}

PlaceResult find_best_placement_flexible(int p, long long rem_time, long long dep_time, bool quick_mode = false) {
    compute_bps();
    PlaceResult best = {{}, -1.0, -1, -1e9, -1e9, 9999, 0};
    
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

    // === Engine 1: 完全長方形 & 近似長方形 (開けた草地用) ===
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
        
        auto get_rect_cells = [](int r, int c, int w, int h) {
            vector<pair<int,int>> cells;
            cells.reserve(w * h);
            for (int i = r; i < r + h; i++)
                for (int j = c; j < c + w; j++)
                    cells.push_back({i, j});
            return cells;
        };
        
        for (auto [w, h] : dims) {
            double comp = 2.0 * sqrt((double)p) / (w + h);
            for (int r = 0; r + h <= N; r++) {
                for (int c = 0; c + w <= N; c++) {
                    if (count_blocked(r, c, h, w) == 0) {
                        auto cells = get_rect_cells(r, c, w, h);
                        double tb = get_tie_breaker(cells, rem_time);
                        int adj = calc_adjacency(cells);
                        int ef = calc_exposed_free(cells);
                        double tbonus = calc_time_bonus(cells, dep_time);
                        PlaceResult cand = {cells, comp, adj, tb, tbonus, ef, 0};
                        if (is_better(cand, best)) best = cand;
                    }
                }
            }
        }
    }
    
    // === Engine 2 & 3: Pure Dynamic Wavefront Growth (障害物・入組みエリア用) ===
    vector<PlaceResult> cands;
    int step = (N <= 30 ? 2 : 3);
    
    for (int i = 0; i < N; i += step) {
        for (int j = 0; j < N; j += step) {
            if (grid[i][j] == '.' && own[i][j] == -1) {
                // Min-Perimeter Growth
                auto reg1 = build_min_perimeter_blob(i, j, p);
                if (!reg1.empty()) {
                    double comp = compactness(reg1);
                    int adj = calc_adjacency(reg1);
                    double tb = get_tie_breaker(reg1, rem_time);
                    int ef = calc_exposed_free(reg1);
                    double tbonus = calc_time_bonus(reg1, dep_time);
                    cands.push_back({reg1, comp, adj, tb, tbonus, ef, 0});
                }
                
                // Nook-Priority Growth
                auto reg2 = build_nook_priority_blob(i, j, p);
                if (!reg2.empty()) {
                    double comp = compactness(reg2);
                    int adj = calc_adjacency(reg2);
                    double tb = get_tie_breaker(reg2, rem_time);
                    int ef = calc_exposed_free(reg2);
                    double tbonus = calc_time_bonus(reg2, dep_time);
                    cands.push_back({reg2, comp, adj, tb, tbonus, ef, 0});
                }
            }
        }
    }
    
    if (!cands.empty()) {
        sort(cands.begin(), cands.end(), [](const PlaceResult& a, const PlaceResult& b) {
            return (a.compact + W_ADJ * a.adj - W_EXPOSE * a.exposed_free) >
                   (b.compact + W_ADJ * b.adj - W_EXPOSE * b.exposed_free);
        });
        if ((int)cands.size() > 10) cands.resize(10);
        
        for (auto& cand : cands) {
            cand.max_free_rect = eval_cap(cand.cells);
            if (is_better(cand, best)) best = cand;
        }
    }
    
    return best;
}

struct MultiMovePlan {
    long long net_gain;
    vector<pair<int,int>> icells;
    vector<int> intersecting;
    vector<pair<int, vector<pair<int,int>>>> move_plans;
};

int size_bucket(int p) {
    if (p <= 30) return 0;
    if (p <= 70) return 1;
    if (p <= 110) return 2;
    return 3;
}

struct Diagnostics {
    int accepted = 0;
    int rejected = 0;
    double total_compact = 0.0;
    int phase1_count = 0;
    int phase2_count = 0;
    int phase25_count = 0;
    int phase3_count = 0;
    int move_count = 0;
    long long total_move_cost = 0;
    long long total_fee = 0;
    
    int bucket_n[4] = {0};
    int bucket_reject[4] = {0};
    double bucket_compact_sum[4] = {0.0};
    long long bucket_fee[4] = {0};
};

Diagnostics diag;

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
        
        double progress = (double)turn / M;
        double current_thresh = (progress < 0.7) ? 0.75 : (0.75 - (progress - 0.7) / 0.3 * 0.45);
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
        
        auto result = find_best_placement_flexible(p, rem_time, grp[turn].t);
        
        if (!result.cells.empty()) {
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
        } else {
            cout << "0\nNo\n";
            diag.rejected++;
            diag.bucket_reject[size_bucket(p)]++;
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
    cerr << "   AHC069 Flexible Non-Rectangle Diagnostics  " << endl;
    cerr << "==============================================" << endl;
    cerr << "Execution Time: " << fixed << setprecision(3) << elapsed << " s / Limit: 2.000s" << endl;
    cerr << "Acceptance: Accepted=" << diag.accepted << " / Rejected=" << diag.rejected 
         << " (Rate: " << fixed << setprecision(1) << 100.0*diag.accepted/max(1, diag.accepted+diag.rejected) << "%)" << endl;
    if (diag.accepted > 0)
        cerr << "Average Compactness: " << fixed << setprecision(4) << diag.total_compact / diag.accepted << endl;
    cerr << "Net Fee: " << diag.total_fee << endl;
    
    return 0;
}
