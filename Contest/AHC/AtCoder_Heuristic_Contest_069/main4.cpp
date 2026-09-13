#include <iostream>
#include <vector>
#include <string>
#include <algorithm>
#include <cmath>
#include <queue>
#include <ctime>
#include <cstring>
#include <iomanip>

using namespace std;

static const int MAXN = 50;
int N, M;
double R;
char grid[MAXN][MAXN];
int own[MAXN][MAXN];
clock_t start_time;

struct GroupInfo {
    int id;
    long long s, t, p, v;
    bool active;
    vector<pair<int,int>> cells;
    double c_min;
} grp[1005];

struct SlotTemplate {
    int r, c, w, h;
    int area;
    int excess;
    int p_target;
    double compact;
    int pond_dist;
};

vector<SlotTemplate> p_templates[155];

int dr[4] = {-1, 1, 0, 0};
int dc[4] = {0, 0, -1, 1};

int count_blocked(int r, int c, int h, int w) {
    int cnt = 0;
    for (int i = r; i < r + h; i++) {
        for (int j = c; j < c + w; j++) {
            if (grid[i][j] == '#' || own[i][j] != -1) cnt++;
        }
    }
    return cnt;
}

vector<pair<int,int>> get_rect_cells(int r, int c, int w, int h, int excess) {
    vector<pair<int,int>> cells;
    cells.reserve(w * h - excess);
    int total = w * h - excess;
    int count = 0;
    
    // 枠内を規則正しくラスタスキャンで取得
    for (int i = r; i < r + h && count < total; i++) {
        for (int j = c; j < c + w && count < total; j++) {
            if (grid[i][j] == '.' && own[i][j] == -1) {
                cells.push_back({i, j});
                count++;
            }
        }
    }
    return (int)cells.size() == total ? cells : vector<pair<int,int>>{};
}

double compactness(const vector<pair<int,int>>& cells) {
    if (cells.empty()) return 0.0;
    int p = cells.size();
    bool in_g[MAXN][MAXN];
    memset(in_g, 0, sizeof(in_g));
    for (const auto& [x, y] : cells) in_g[x][y] = true;
    
    int shared = 0;
    for (const auto& [x, y] : cells) {
        for (int d = 0; d < 4; d++) {
            int nx = x + dr[d], ny = y + dc[d];
            if (nx >= 0 && nx < N && ny >= 0 && ny < N && in_g[nx][ny]) shared++;
        }
    }
    int L = 4 * p - shared;
    return (4.0 * sqrt((double)p)) / (double)L;
}

void precompute_all_slot_templates() {
    for (int p = 4; p <= 150; p++) {
        int sq = max(2, (int)round(sqrt((double)p)));
        vector<SlotTemplate> cands;
        
        for (int w = sq; w <= min(N, sq + 5); w++) {
            for (int h = max(2, (p + w - 1) / w); h <= min(N, w + 3); h++) {
                int area = w * h;
                int excess = area - p;
                if (excess < 0 || excess > min(w, h)) continue;
                
                for (int r = 0; r + h <= N; r++) {
                    for (int c = 0; c + w <= N; c++) {
                        if (count_blocked(r, c, h, w) == 0) {
                            auto cells = get_rect_cells(r, c, w, h, excess);
                            if (!cells.empty()) {
                                double comp = compactness(cells);
                                cands.push_back({r, c, w, h, area, excess, p, comp, 0});
                            }
                        }
                    }
                }
            }
        }
        
        sort(cands.begin(), cands.end(), [](const SlotTemplate& a, const SlotTemplate& b) {
            return a.compact > b.compact;
        });
        
        if ((int)cands.size() > 15) cands.resize(15);
        p_templates[p] = cands;
    }
}

struct Diag {
    int accepted = 0, rejected = 0;
    int filter_rejected = 0, no_place_rejected = 0;
    double total_occupancy_sum = 0.0;
    double q_occupancy_sum[4] = {};
    int q_count[4] = {};
} diag;

int size_bucket(int p) {
    if (p <= 30) return 0;
    if (p <= 70) return 1;
    if (p <= 110) return 2;
    return 3;
}

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
    
    memset(own, -1, sizeof(own));
    precompute_all_slot_templates();
    
    double global_total_density = 0;
    int global_density_count = 0;
    double global_avg_density = 0;
    
    for (int turn = 0; turn < M; turn++) {
        int id;
        long long s, t, p, v;
        if (!(cin >> id >> s >> t >> p >> v)) break;
        
        grp[turn] = {id, s, t, p, v, true, {}, 1.0};
        long long rem_time = t - s;
        long long now = s;
        
        double density = (double)v / ((double)p * rem_time);
        global_total_density += density;
        global_density_count++;
        global_avg_density = global_total_density / global_density_count;
        
        int cur_free = 0;
        for (int i = 0; i < N; i++)
            for (int j = 0; j < N; j++)
                if (grid[i][j] == '.' && own[i][j] == -1) cur_free++;
        double free_rate = (double)cur_free / total_grass;
        double occupancy = 1.0 - free_rate;
        diag.total_occupancy_sum += occupancy;
        int q = min(3, turn / 250);
        diag.q_occupancy_sum[q] += occupancy;
        diag.q_count[q]++;
        
        // 退去処理
        for (int j = 0; j < turn; j++) {
            if (grp[j].active && grp[j].t < now) {
                for (const auto& [x, y] : grp[j].cells) own[x][y] = -1;
                grp[j].cells.clear();
                grp[j].active = false;
            }
        }
        
        // 密度事前選別
        double base_thresh = 0.28 + 0.50 * (1.0 - free_rate);
        if (p >= 90) base_thresh = 0.45 + 0.60 * (1.0 - free_rate);
        else if (p <= 30) base_thresh = 0.15 + 0.35 * (1.0 - free_rate);
        
        bool is_vip = (v >= 45000000LL) || (turn >= M - 40);
        double current_thresh = is_vip ? 0.0 : base_thresh;
        
        if (global_density_count > 20 && density < global_avg_density * current_thresh) {
            cout << "0\nNo\n" << flush;
            diag.rejected++;
            diag.filter_rejected++;
            continue;
        }
        
        // 事前型枠スロットからのハメ込み探索
        vector<pair<int,int>> best_cells;
        double best_compact = -1.0;
        
        for (const auto& tmpl : p_templates[p]) {
            if (count_blocked(tmpl.r, tmpl.c, tmpl.h, tmpl.w) == 0) {
                auto cells = get_rect_cells(tmpl.r, tmpl.c, tmpl.w, tmpl.h, tmpl.excess);
                if (!cells.empty()) {
                    double comp = compactness(cells);
                    if (comp > best_compact) {
                        best_compact = comp;
                        best_cells = cells;
                    }
                }
            }
        }
        
        if (!best_cells.empty()) {
            cout << "0\nYes\n";
            for (const auto& [x, y] : best_cells) {
                cout << x << " " << y << "\n";
                own[x][y] = turn;
            }
            grp[turn].cells = best_cells;
            grp[turn].c_min = best_compact;
            grp[turn].active = true;
            diag.accepted++;
        } else {
            cout << "0\nNo\n";
            diag.rejected++;
            diag.no_place_rejected++;
        }
        cout << flush;
    }
    
    double elapsed = (double)(clock() - start_time) / CLOCKS_PER_SEC;
    cerr << "\n==============================================" << endl;
    cerr << "   AHC069 main4.cpp Diagnostics Report" << endl;
    cerr << "==============================================" << endl;
    cerr << "Execution Time: " << fixed << setprecision(3) << elapsed << " s" << endl;
    cerr << "Acceptance: Accepted=" << diag.accepted << " / Rejected=" << diag.rejected 
         << " (FilterRej=" << diag.filter_rejected << ", NoPlaceRej=" << diag.no_place_rejected << ")"
         << " (Rate: " << fixed << setprecision(1) << 100.0*diag.accepted/max(1, diag.accepted+diag.rejected) << "%)" << endl;
    cerr << "Occupancy: Overall=" << fixed << setprecision(2) << 100.0 * diag.total_occupancy_sum / M << "%"
         << " | Q1=" << 100.0 * diag.q_occupancy_sum[0] / max(1, diag.q_count[0]) << "%"
         << " | Q2=" << 100.0 * diag.q_occupancy_sum[1] / max(1, diag.q_count[1]) << "%"
         << " | Q3=" << 100.0 * diag.q_occupancy_sum[2] / max(1, diag.q_count[2]) << "%"
         << " | Q4=" << 100.0 * diag.q_occupancy_sum[3] / max(1, diag.q_count[3]) << "%" << endl;
         
    return 0;
}
