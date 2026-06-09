#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>
#include <chrono>
#include <random>
#include <cmath>
#include <unordered_set>

using namespace std;

// 方向: 0=右, 1=下, 2=左, 3=上
const int dr[] = {0, 1, 0, -1};
const int dc[] = {1, 0, -1, 0};

int N, M, T;
vector<string> v_walls, h_walls;

struct State {
    int r, c, d;
};

struct Point {
    int r, c;
};

// グローバルに配置（評価関数からアクセスするため）
vector<Point> balls, baskets;
int dist_apsp[20][20][4][20][20][4];

// 指定した方向に壁がなく、盤面内に収まっているか判定
bool can_move(int r, int c, int d) {
    if (d == 0) {
        return (c + 1 < N && v_walls[r][c] == '0');
    } else if (d == 1) {
        return (r + 1 < N && h_walls[r][c] == '0');
    } else if (d == 2) {
        return (c - 1 >= 0 && v_walls[r][c - 1] == '0');
    } else if (d == 3) {
        return (r - 1 >= 0 && h_walls[r - 1][c] == '0');
    }
    return false;
}

// すべての (r, c, d) のペア間の最短距離を事前計算 (BFS)
void precompute_apsp() {
    for (int r = 0; r < N; ++r) {
        for (int c = 0; c < N; ++c) {
            for (int d = 0; d < 4; ++d) {
                // 初期化
                for (int i = 0; i < N; ++i)
                    for (int j = 0; j < N; ++j)
                        for (int k = 0; k < 4; ++k)
                            dist_apsp[r][c][d][i][j][k] = 1e9;
                
                dist_apsp[r][c][d][r][c][d] = 0;
                queue<State> q;
                q.push({r, c, d});
                
                while (!q.empty()) {
                    State u = q.front();
                    q.pop();
                    int cur_dist = dist_apsp[r][c][d][u.r][u.c][u.d];
                    
                    // F (前進)
                    if (can_move(u.r, u.c, u.d)) {
                        int nr = u.r + dr[u.d];
                        int nc = u.c + dc[u.d];
                        if (dist_apsp[r][c][d][nr][nc][u.d] > cur_dist + 1) {
                            dist_apsp[r][c][d][nr][nc][u.d] = cur_dist + 1;
                            q.push({nr, nc, u.d});
                        }
                    }
                    // R (右折)
                    int nd = (u.d + 1) % 4;
                    if (dist_apsp[r][c][d][u.r][u.c][nd] > cur_dist + 1) {
                        dist_apsp[r][c][d][u.r][u.c][nd] = cur_dist + 1;
                        q.push({u.r, u.c, nd});
                    }
                    // L (左折)
                    nd = (u.d + 3) % 4;
                    if (dist_apsp[r][c][d][u.r][u.c][nd] > cur_dist + 1) {
                        dist_apsp[r][c][d][u.r][u.c][nd] = cur_dist + 1;
                        q.push({u.r, u.c, nd});
                    }
                }
            }
        }
    }
}

// 順列 P に従った場合の総コストを DP で高速に評価
int evaluate(const vector<int>& P) {
    int dp[4] = {0, (int)1e9, (int)1e9, (int)1e9}; // 初期状態は右向き(0)
    int curr_r = 0, curr_c = 0;
    
    for (int i = 0; i < M; ++i) {
        int b = P[i];
        
        // ボールへ移動
        int next_dp[4] = {(int)1e9, (int)1e9, (int)1e9, (int)1e9};
        for (int pd = 0; pd < 4; ++pd) {
            if (dp[pd] > 1e8) continue;
            for (int nd = 0; nd < 4; ++nd) {
                int cost = dist_apsp[curr_r][curr_c][pd][balls[b].r][balls[b].c][nd];
                next_dp[nd] = min(next_dp[nd], dp[pd] + cost + 1); // +1 は 'S' 操作
            }
        }
        for (int d = 0; d < 4; ++d) dp[d] = next_dp[d];
        curr_r = balls[b].r;
        curr_c = balls[b].c;
        
        // かごへ移動
        int next_dp2[4] = {(int)1e9, (int)1e9, (int)1e9, (int)1e9};
        for (int pd = 0; pd < 4; ++pd) {
            if (dp[pd] > 1e8) continue;
            for (int nd = 0; nd < 4; ++nd) {
                int cost = dist_apsp[curr_r][curr_c][pd][baskets[b].r][baskets[b].c][nd];
                next_dp2[nd] = min(next_dp2[nd], dp[pd] + cost + 1); // +1 は 'S' 操作
            }
        }
        for (int d = 0; d < 4; ++d) dp[d] = next_dp2[d];
        curr_r = baskets[b].r;
        curr_c = baskets[b].c;
    }
    
    int ans = 1e9;
    for (int d = 0; d < 4; ++d) ans = min(ans, dp[d]);
    return ans;
}

// （ベースラインと同じ）現在地からターゲットへの最短操作列をローカルBFSで求め、状態を更新する
string get_path(int &curr_r, int &curr_c, int &curr_d, int target_r, int target_c) {
    vector<vector<vector<int>>> dist(N, vector<vector<int>>(N, vector<int>(4, 1e9)));
    vector<vector<vector<char>>> move_char(N, vector<vector<char>>(N, vector<char>(4, ' ')));
    vector<vector<vector<State>>> prev(N, vector<vector<State>>(N, vector<State>(4)));

    queue<State> q;
    q.push({curr_r, curr_c, curr_d});
    dist[curr_r][curr_c][curr_d] = 0;

    while (!q.empty()) {
        State u = q.front();
        q.pop();

        if (can_move(u.r, u.c, u.d)) {
            int nr = u.r + dr[u.d], nc = u.c + dc[u.d];
            if (dist[nr][nc][u.d] > dist[u.r][u.c][u.d] + 1) {
                dist[nr][nc][u.d] = dist[u.r][u.c][u.d] + 1;
                prev[nr][nc][u.d] = u;
                move_char[nr][nc][u.d] = 'F';
                q.push({nr, nc, u.d});
            }
        }
        int nd = (u.d + 1) % 4;
        if (dist[u.r][u.c][nd] > dist[u.r][u.c][u.d] + 1) {
            dist[u.r][u.c][nd] = dist[u.r][u.c][u.d] + 1;
            prev[u.r][u.c][nd] = u;
            move_char[u.r][u.c][nd] = 'R';
            q.push({u.r, u.c, nd});
        }
        nd = (u.d + 3) % 4;
        if (dist[u.r][u.c][nd] > dist[u.r][u.c][u.d] + 1) {
            dist[u.r][u.c][nd] = dist[u.r][u.c][u.d] + 1;
            prev[u.r][u.c][nd] = u;
            move_char[u.r][u.c][nd] = 'L';
            q.push({u.r, u.c, nd});
        }
    }

    int best_d = 0, min_dist = 1e9;
    for (int i = 0; i < 4; ++i) {
        if (dist[target_r][target_c][i] < min_dist) {
            min_dist = dist[target_r][target_c][i];
            best_d = i;
        }
    }

    string path = "";
    State curr = {target_r, target_c, best_d};
    while (curr.r != curr_r || curr.c != curr_c || curr.d != curr_d) {
        path += move_char[curr.r][curr.c][curr.d];
        curr = prev[curr.r][curr.c][curr.d];
    }
    reverse(path.begin(), path.end());

    curr_r = target_r; curr_c = target_c; curr_d = best_d;
    return path;
}

// 焼きなまし法で得られた操作列 s を受け取り、最適に圧縮した文字列を返す
string compress_string(const string& s) {
    int n = s.length();
    int best_len = 0;
    string best_sub = "";
    int max_saved = 0;
    
    unordered_set<string> seen;

    // マクロの長さを探索（長すぎても出現回数が減るため、2〜40程度で十分）
    for (int len = 2; len <= 40; ++len) {
        seen.clear();
        for (int i = 0; i <= n - len; ++i) {
            string sub = s.substr(i, len);
            
            // 既に評価済みの部分文字列ならスキップ（高速化）
            if (seen.count(sub)) continue;
            seen.insert(sub);
            
            int count = 0;
            int j = 0;
            // 重複しない出現回数をカウント
            while (j <= n - len) {
                if (s.compare(j, len, sub) == 0) {
                    count++;
                    j += len;
                } else {
                    j++;
                }
            }
            
            // 削減量 = (通常時の総操作数) - (マクロ使用時の総操作数)
            // = (len * count) - (len + 2 + count - 1)
            // = (len - 1) * (count - 1) - 2
            int saved = (len - 1) * (count - 1) - 2;
            
            if (saved > max_saved) {
                max_saved = saved;
                best_sub = sub;
                best_len = len;
            }
        }
    }

    // 圧縮効果がない場合はそのまま返す
    if (max_saved <= 0) return s;

    // 最適な部分文字列をマクロ化して適用
    string res = "";
    bool first = true;
    int i = 0;
    while (i < n) {
        if (i <= n - best_len && s.compare(i, best_len, best_sub) == 0) {
            if (first) {
                res += "M" + best_sub + "M"; // 初回は記録
                first = false;
            } else {
                res += "P"; // 2回目以降は再生
            }
            i += best_len;
        } else {
            res += s[i];
            i++;
        }
    }
    return res;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    cin >> N >> M >> T;
    
    v_walls.resize(N);
    for (int i = 0; i < N; ++i) cin >> v_walls[i];
    
    h_walls.resize(N - 1);
    for (int i = 0; i < N - 1; ++i) cin >> h_walls[i];

    balls.resize(M);
    baskets.resize(M);
    for (int i = 0; i < M; ++i) {
        cin >> balls[i].r >> balls[i].c >> baskets[i].r >> baskets[i].c;
    }

    // 1. 全点対最短経路の事前計算
    precompute_apsp();

    // 2. 焼きなまし法による回収順序の最適化
    vector<int> P(M);
    for (int i = 0; i < M; ++i) P[i] = i;

    int current_score = evaluate(P);
    vector<int> best_P = P;
    int best_score = current_score;

    auto start_time = chrono::system_clock::now();
    double MAX_TIME = 1.8; // 実行時間制限マージン

    mt19937 mt(42);
    double start_temp = 10.0;
    double end_temp = 0.01;

    int iter = 0;
    while (true) {
        if ((iter & 255) == 0) {
            auto current_time = chrono::system_clock::now();
            double elapsed = chrono::duration_cast<chrono::duration<double>>(current_time - start_time).count();
            if (elapsed > MAX_TIME) break;
        }
        iter++;

        int type = mt() % 3;
        int i = mt() % M;
        int j = mt() % M;
        if (i == j) continue;

        vector<int> next_P = P;
        if (type == 0) {
            // Swap
            swap(next_P[i], next_P[j]);
        } else if (type == 1) {
            // Reverse
            if (i > j) swap(i, j);
            reverse(next_P.begin() + i, next_P.begin() + j + 1);
        } else {
            // Insert
            int val = next_P[i];
            next_P.erase(next_P.begin() + i);
            next_P.insert(next_P.begin() + j, val);
        }

        int next_score = evaluate(next_P);
        
        auto current_time = chrono::system_clock::now();
        double elapsed = chrono::duration_cast<chrono::duration<double>>(current_time - start_time).count();
        double temp = start_temp + (end_temp - start_temp) * (elapsed / MAX_TIME);

        // 遷移の受容判定
        if (next_score <= current_score || exp((current_score - next_score) / temp) > (double)(mt() % 10000) / 10000.0) {
            P = next_P;
            current_score = next_score;
            if (current_score < best_score) {
                best_score = current_score;
                best_P = P;
            }
        }
    }

    // 3. 最良の順列を使って操作列を生成
    int curr_r = 0, curr_c = 0, curr_d = 0;
    vector<char> ans;

    for (int i = 0; i < M; ++i) {
        int b = best_P[i];
        
        string p1 = get_path(curr_r, curr_c, curr_d, balls[b].r, balls[b].c);
        for (char c : p1) ans.push_back(c);
        ans.push_back('S');

        string p2 = get_path(curr_r, curr_c, curr_d, baskets[b].r, baskets[b].c);
        for (char c : p2) ans.push_back(c);
        ans.push_back('S');
    }

    // 4. マクロによる操作列の圧縮と結果の出力
    string base_ans = "";
    for (char c : ans) base_ans += c;

    // マクロによる圧縮を適用
    string final_ans = compress_string(base_ans);

    int output_count = 0;
    for (char c : final_ans) {
        if (output_count >= T) break;
        cout << c << "\n";
        output_count++;
    }

    return 0;
}