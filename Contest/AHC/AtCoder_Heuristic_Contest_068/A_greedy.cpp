#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <algorithm>
#include <chrono>
#include <random>

using namespace std;

const int N = 20;

struct Operation {
    char dir; 
    int r, c, h, w;
};

int a[N][N];
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

    int n_in;
    cin >> n_in;

    for (int i = 0; i < N; ++i) {
        for (int j = 0; j < N; ++j) {
            cin >> a[i][j];
        }
    }
    for (int i = 0; i < N; ++i) cin >> V[i];
    for (int i = 0; i < N - 1; ++i) cin >> H[i];

    // 有効操作の列挙（約数千〜数万個）
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

    vector<Operation> ans;
    int current_dist = calc_total_distance();

    cerr << "[Construct] Starting Whole-Rect Evaluated True Greedy..." << endl;

    // 毎ターン、すべての有効な長方形からベストなものを厳密に選ぶ
    while (ans.size() < 100000) {
        Operation best_op;
        long long best_score = -999999;
        int best_dist_diff = 0;

        // 全ての有効な操作を走査してブロック全体の利益を評価
        for (const auto& op : valid_ops) {
            int before_d = 0; int before_m = 0;
            for (int r = op.r; r < op.r + op.h; ++r) {
                for (int c = op.c; c < op.c + op.w; ++c) {
                    if (a[r][c] == r * N + c) before_m++;
                    int tr = a[r][c] / N; int tc = a[r][c] % N;
                    before_d += abs(r - tr) + abs(c - tc);
                }
            }

            apply_op(op); // 試しに適用

            int after_d = 0; int after_m = 0;
            for (int r = op.r; r < op.r + op.h; ++r) {
                for (int c = op.c; c < op.c + op.w; ++c) {
                    if (a[r][c] == r * N + c) after_m++;
                    int tr = a[r][c] / N; int tc = a[r][c] % N;
                    after_d += abs(r - tr) + abs(c - tc);
                }
            }

            apply_op(op); // 元に戻す

            int dist_diff = before_d - after_d; // 正なら距離が縮まる
            int matched_diff = after_m - before_m; // 正なら正解マスが増える、負なら揃っていたマスを破壊

            // 【長方形全体の総合評価スコア】
            // 距離の改善をベースにしつつ、揃ったマスを増やす手にボーナス、壊す手に重いペナルティ
            long long op_score = (long long)dist_diff * 1;
            if (matched_diff > 0) op_score += (long long)matched_diff * 50;
            if (matched_diff < 0) op_score += (long long)matched_diff * 200;

            if (op_score > best_score) {
                best_score = op_score;
                best_op = op;
                best_dist_diff = dist_diff;
            }
        }

        // 盤面全体でスコアを改善できる手が1つもなくなったら、本当の局所最適解として終了
        if (best_score <= 0) {
            cerr << "[Stop] Reached Local Optimum. No improving operations found." << endl;
            break;
        }

        // 【デバッグ出力】毎ターンの詳細な挙動を追跡
        int matched_before = calc_matched_cells();
        apply_op(best_op);
        int matched_after = calc_matched_cells();
        
        current_dist -= best_dist_diff;
        ans.push_back(best_op);

        cerr << "Turn=" << ans.size() 
             << " | Score=" << best_score 
             << " | Dist=" << current_dist 
             << " | Matched: " << matched_before << " -> " << matched_after << endl;
    }

    cerr << "[Final Debug] Total Ops: " << ans.size() 
         << " | Final Dist: " << current_dist 
         << " | Final Matched: " << calc_matched_cells() << endl;

    for (const auto& op : ans) {
        cout << op.dir << " " << op.r << " " << op.c << " " << op.h << " " << op.w << "\n";
    }
    cout << flush;

    return 0;
}