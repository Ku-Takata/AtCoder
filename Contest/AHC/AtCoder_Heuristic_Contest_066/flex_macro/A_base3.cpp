#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <algorithm>
#include <queue>
#include <unordered_map>

using namespace std;

struct Point {
    int r, c;
    bool operator==(const Point& other) const { return r == other.r && c == other.c; }
};

struct Ball {
    Point start;
    Point goal;
};

// DP用の状態管理構造体
struct StateValue {
    int cost;          // ここまでの操作文字数
    int prev_i;        // 遷移元の index
    string prev_macro; // 遷移元の登録マクロ
    int action_type;   // 0: そのまま出力, 1: Pで再生, 2: Mで新規登録
    
    StateValue() : cost(1e9), prev_i(-1), prev_macro(""), action_type(-1) {}
    StateValue(int c, int pi, string pm, int a) 
        : cost(c), prev_i(pi), prev_macro(pm), action_type(a) {}
};

class Solver {
    int N, M, T;
    vector<string> v; 
    vector<string> h; 
    vector<Ball> balls;

public:
    Solver() {
        cin >> N >> M >> T;
        v.resize(N);
        for (int i = 0; i < N; ++i) cin >> v[i];
        
        h.resize(N - 1); 
        for (int i = 0; i < N - 1; ++i) cin >> h[i];

        balls.resize(M);
        for (int i = 0; i < M; ++i) {
            cin >> balls[i].start.r >> balls[i].start.c 
                >> balls[i].goal.r >> balls[i].goal.c;
        }
    }

    void solve() {
        string raw_commands = generate_raw_path();
        string compressed_commands = compress_commands(raw_commands);
        
        // 制約に従い、T回を超える操作は無効なので切り捨てる
        if (compressed_commands.length() > T) {
            compressed_commands = compressed_commands.substr(0, T);
        }
        
        for (char c : compressed_commands) {
            cout << c << "\n";
        }
    }

private:
    struct State { int r, c, dir; };

    pair<string, int> get_path(int sr, int sc, int sdir, int gr, int gc) {
        if (sr == gr && sc == gc) return {"", sdir};
        
        vector<vector<vector<int>>> dist(N, vector<vector<int>>(N, vector<int>(4, 1e9)));
        vector<vector<vector<State>>> prev(N, vector<vector<State>>(N, vector<State>(4, {-1, -1, -1})));
        vector<vector<vector<char>>> cmd(N, vector<vector<char>>(N, vector<char>(4, ' ')));

        queue<State> q;
        q.push({sr, sc, sdir});
        dist[sr][sc][sdir] = 0;

        int dr[] = {0, 1, 0, -1}; 
        int dc[] = {1, 0, -1, 0};

        while(!q.empty()){
            auto u = q.front(); q.pop();
            
            int nr = u.r + dr[u.dir];
            int nc = u.c + dc[u.dir];
            bool can_move = false;
            if(nr >= 0 && nr < N && nc >= 0 && nc < N) {
                can_move = true;
                if(u.dir == 0 && v[u.r][u.c] == '1') can_move = false;
                if(u.dir == 1 && h[u.r][u.c] == '1') can_move = false;
                if(u.dir == 2 && v[u.r][u.c - 1] == '1') can_move = false;
                if(u.dir == 3 && h[u.r - 1][u.c] == '1') can_move = false;
            }
            if(can_move && dist[nr][nc][u.dir] > dist[u.r][u.c][u.dir] + 1) {
                dist[nr][nc][u.dir] = dist[u.r][u.c][u.dir] + 1;
                prev[nr][nc][u.dir] = u;
                cmd[nr][nc][u.dir] = 'F';
                q.push({nr, nc, u.dir});
            }

            int ndirR = (u.dir + 1) % 4;
            if(dist[u.r][u.c][ndirR] > dist[u.r][u.c][u.dir] + 1) {
                dist[u.r][u.c][ndirR] = dist[u.r][u.c][u.dir] + 1;
                prev[u.r][u.c][ndirR] = u;
                cmd[u.r][u.c][ndirR] = 'R';
                q.push({u.r, u.c, ndirR});
            }

            int ndirL = (u.dir + 3) % 4;
            if(dist[u.r][u.c][ndirL] > dist[u.r][u.c][u.dir] + 1) {
                dist[u.r][u.c][ndirL] = dist[u.r][u.c][u.dir] + 1;
                prev[u.r][u.c][ndirL] = u;
                cmd[u.r][u.c][ndirL] = 'L';
                q.push({u.r, u.c, ndirL});
            }
        }

        int best_dir = -1;
        int min_d = 1e9;
        for(int d = 0; d < 4; ++d) {
            if(dist[gr][gc][d] < min_d) {
                min_d = dist[gr][gc][d];
                best_dir = d;
            }
        }

        if(best_dir == -1) return {"", sdir}; 

        string res = "";
        State curr = {gr, gc, best_dir};
        while(!(curr.r == sr && curr.c == sc && curr.dir == sdir)) {
            res += cmd[curr.r][curr.c][curr.dir];
            curr = prev[curr.r][curr.c][curr.dir];
        }
        reverse(res.begin(), res.end());
        return {res, best_dir};
    }

    string generate_raw_path() {
        string commands = "";
        int r = 0, c = 0, dir = 0; 

        for (int i = 0; i < M; ++i) {
            auto p1 = get_path(r, c, dir, balls[i].start.r, balls[i].start.c);
            commands += p1.first;
            commands += "S";
            r = balls[i].start.r;
            c = balls[i].start.c;
            dir = p1.second;

            auto p2 = get_path(r, c, dir, balls[i].goal.r, balls[i].goal.c);
            commands += p2.first;
            commands += "S";
            r = balls[i].goal.r;
            c = balls[i].goal.c;
            dir = p2.second;
        }
        
        return commands;
    }

    // フェーズ2: 戦略3（ビームサーチによるグローバル最適化）
    string compress_commands(const string& raw) {
        int n = raw.length();
        if (n == 0) return "";

        // パラメータ：ビーム幅とマクロの最大長。
        // TLEになる場合はBEAM_WIDTHを下げる、余裕があれば上げる。
        const int BEAM_WIDTH = 20; 
        const int MAX_MACRO_LEN = 30; 

        // dp[i][macro_string] = StateValue
        // i文字目までを処理した時点で、レジスタにmacro_stringが入っている状態の最小コスト
        vector<unordered_map<string, StateValue>> dp(n + 1);
        dp[0][""] = StateValue(0, -1, "", -1);

        for (int i = 0; i < n; ++i) {
            if (dp[i].empty()) continue;

            // 状態空間の爆発を防ぐため、コストが低い上位BEAM_WIDTH個だけを残す
            vector<pair<string, StateValue>> states(dp[i].begin(), dp[i].end());
            if (states.size() > BEAM_WIDTH) {
                nth_element(states.begin(), states.begin() + BEAM_WIDTH, states.end(),
                    [](const pair<string, StateValue>& a, const pair<string, StateValue>& b) {
                        if (a.second.cost != b.second.cost)
                            return a.second.cost < b.second.cost;
                        // コストが同じ場合は、長いマクロを持っている方を優先
                        return a.first.length() > b.first.length(); 
                    });
                states.resize(BEAM_WIDTH);
            }

            for (const auto& st : states) {
                const string& m = st.first;
                int c = st.second.cost;

                // 遷移1: 1文字そのまま出力
                int nxt1 = i + 1;
                if (c + 1 < dp[nxt1][m].cost) {
                    dp[nxt1][m] = StateValue(c + 1, i, m, 0);
                }

                // 遷移2: 登録済みのマクロを再生 (P)
                if (!m.empty() && i + m.length() <= n) {
                    bool match = true;
                    for (size_t j = 0; j < m.length(); ++j) {
                        if (raw[i + j] != m[j]) { match = false; break; }
                    }
                    if (match) {
                        int nxt2 = i + m.length();
                        if (c + 1 < dp[nxt2][m].cost) {
                            dp[nxt2][m] = StateValue(c + 1, i, m, 1);
                        }
                    }
                }

                // 遷移3: 新しいマクロを登録 (M ... M)
                for (int L = 2; L <= MAX_MACRO_LEN; ++L) {
                    if (i + L <= n) {
                        string new_m = raw.substr(i, L);
                        int nxt3 = i + L;
                        // M(1) + L文字 + M(1) = L + 2コスト
                        if (c + L + 2 < dp[nxt3][new_m].cost) {
                            dp[nxt3][new_m] = StateValue(c + L + 2, i, m, 2);
                        }
                    }
                }
            }
        }

        // --- バックトラッキング（経路復元） ---
        int min_cost = 1e9;
        string best_macro = "";
        
        // 終点（n文字目）に到達した状態のうち、最もコストが小さいものを探す
        for (const auto& kv : dp[n]) {
            if (kv.second.cost < min_cost) {
                min_cost = kv.second.cost;
                best_macro = kv.first;
            }
        }

        vector<string> parts;
        int curr_i = n;
        string curr_m = best_macro;

        // 後ろから前へ操作を辿る
        while (curr_i > 0) {
            StateValue sv = dp[curr_i][curr_m];
            if (sv.action_type == 0) {
                parts.push_back(string(1, raw[curr_i - 1]));
            } else if (sv.action_type == 1) {
                parts.push_back("P");
            } else if (sv.action_type == 2) {
                parts.push_back("M" + curr_m + "M");
            }
            curr_i = sv.prev_i;
            curr_m = sv.prev_macro;
        }

        // 逆順になっているので反転して結合
        reverse(parts.begin(), parts.end());
        string res = "";
        for (const string& p : parts) {
            res += p;
        }

        return res;
    }
};

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);
    Solver solver;
    solver.solve();
    return 0;
}