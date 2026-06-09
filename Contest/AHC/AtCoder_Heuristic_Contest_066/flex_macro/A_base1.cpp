#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <algorithm>
#include <queue> // BFS用に<queue>を追加

using namespace std;

struct Point {
    int r, c;
    bool operator==(const Point& other) const { return r == other.r && c == other.c; }
};

struct Ball {
    Point start;
    Point goal;
};

class Solver {
    int N, M, T;
    vector<string> v; // 縦方向の壁 (v[i][j] == '1' なら (i,j) と (i,j+1) の間に壁)
    vector<string> h; // 横方向の壁 (h[i][j] == '1' なら (i,j) と (i+1,j) の間に壁)
    vector<Ball> balls;

public:
    Solver() {
        cin >> N >> M >> T;
        v.resize(N);
        for (int i = 0; i < N; ++i) cin >> v[i];
        
        h.resize(N - 1); // 問題文の入力形式上、h は N-1 行と推測されます
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
        
        // 出力
        for (char c : compressed_commands) {
            cout << c << "\n";
        }
    }

private:
    struct State { int r, c, dir; };

    // BFSで特定の座標・向きから目的の座標への最短操作列を探索
    pair<string, int> get_path(int sr, int sc, int sdir, int gr, int gc) {
        if (sr == gr && sc == gc) return {"", sdir};
        
        vector<vector<vector<int>>> dist(N, vector<vector<int>>(N, vector<int>(4, 1e9)));
        vector<vector<vector<State>>> prev(N, vector<vector<State>>(N, vector<State>(4, {-1, -1, -1})));
        vector<vector<vector<char>>> cmd(N, vector<vector<char>>(N, vector<char>(4, ' ')));

        queue<State> q;
        q.push({sr, sc, sdir});
        dist[sr][sc][sdir] = 0;

        int dr[] = {0, 1, 0, -1}; // 0:右, 1:下, 2:左, 3:上
        int dc[] = {1, 0, -1, 0};

        while(!q.empty()){
            auto u = q.front(); q.pop();
            
            // 1. 前進 (F)
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

            // 2. 右折 (R)
            int ndirR = (u.dir + 1) % 4;
            if(dist[u.r][u.c][ndirR] > dist[u.r][u.c][u.dir] + 1) {
                dist[u.r][u.c][ndirR] = dist[u.r][u.c][u.dir] + 1;
                prev[u.r][u.c][ndirR] = u;
                cmd[u.r][u.c][ndirR] = 'R';
                q.push({u.r, u.c, ndirR});
            }

            // 3. 左折 (L)
            int ndirL = (u.dir + 3) % 4;
            if(dist[u.r][u.c][ndirL] > dist[u.r][u.c][u.dir] + 1) {
                dist[u.r][u.c][ndirL] = dist[u.r][u.c][u.dir] + 1;
                prev[u.r][u.c][ndirL] = u;
                cmd[u.r][u.c][ndirL] = 'L';
                q.push({u.r, u.c, ndirL});
            }
        }

        // 最短到達時の向きを探す
        int best_dir = -1;
        int min_d = 1e9;
        for(int d = 0; d < 4; ++d) {
            if(dist[gr][gc][d] < min_d) {
                min_d = dist[gr][gc][d];
                best_dir = d;
            }
        }

        if(best_dir == -1) return {"", sdir}; // 物理的に到達不能な場合

        // 経路の復元
        string res = "";
        State curr = {gr, gc, best_dir};
        while(!(curr.r == sr && curr.c == sc && curr.dir == sdir)) {
            res += cmd[curr.r][curr.c][curr.dir];
            curr = prev[curr.r][curr.c][curr.dir];
        }
        reverse(res.begin(), res.end());
        return {res, best_dir};
    }

    // フェーズ1: 愚直な操作列（F, R, L, Sのみ）を生成する
    string generate_raw_path() {
        string commands = "";
        int r = 0, c = 0, dir = 0; // 初期位置 (0,0) で右を向いている

        for (int i = 0; i < M; ++i) {
            // ボールを取りに行く
            auto p1 = get_path(r, c, dir, balls[i].start.r, balls[i].start.c);
            commands += p1.first;
            commands += "S";
            r = balls[i].start.r;
            c = balls[i].start.c;
            dir = p1.second;

            // かごへ置きに行く
            auto p2 = get_path(r, c, dir, balls[i].goal.r, balls[i].goal.c);
            commands += p2.first;
            commands += "S";
            r = balls[i].goal.r;
            c = balls[i].goal.c;
            dir = p2.second;
        }
        
        return commands;
    }

    // フェーズ2: マクロ機能（M, P）を使って操作列を圧縮する
    string compress_commands(const string& raw) {
        string res = "";
        string current_macro = "";
        int n = raw.length();
        int i = 0;

        const int LOOKAHEAD_LIMIT = 500; 
        const int MAX_MACRO_LEN = 50;

        while (i < n) {
            int search_end = min(n, i + LOOKAHEAD_LIMIT);
            
            int saving_current = 0;
            if (!current_macro.empty()) {
                int pos = i;
                while (pos + current_macro.length() <= search_end) {
                    if (raw.substr(pos, current_macro.length()) == current_macro) {
                        saving_current += current_macro.length() - 1;
                        pos += current_macro.length();
                    } else {
                        pos++;
                    }
                }
            }

            int best_saving_new = -1;
            string best_new_macro = "";
            int max_len = min(MAX_MACRO_LEN, search_end - i);
            
            for (int len = 2; len <= max_len; ++len) {
                string sub = raw.substr(i, len);
                
                int count = 0;
                int pos = i;
                while (pos + len <= search_end) {
                    if (raw.substr(pos, len) == sub) {
                        count++;
                        pos += len;
                    } else {
                        pos++;
                    }
                }
                
                int saving = (len * count) - (len + count + 1);
                
                if (saving > best_saving_new) {
                    best_saving_new = saving;
                    best_new_macro = sub;
                }
            }

            if (best_saving_new > 0 && best_saving_new > saving_current) {
                res += "M" + best_new_macro + "M";
                current_macro = best_new_macro;
                i += best_new_macro.length();
            } 
            else if (!current_macro.empty() && i + current_macro.length() <= n 
                     && raw.substr(i, current_macro.length()) == current_macro) {
                res += "P";
                i += current_macro.length();
            } 
            else {
                res += raw[i];
                i++;
            }
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