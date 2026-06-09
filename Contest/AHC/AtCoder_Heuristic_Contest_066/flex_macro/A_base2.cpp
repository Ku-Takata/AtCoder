#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <algorithm>
#include <queue> 

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
        
        // 【安全装置】問題の制約に従い、T回を超える操作は無効なので切り捨てる
        if (compressed_commands.length() > T) {
            compressed_commands = compressed_commands.substr(0, T);
        }
        
        // 出力
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

    string compress_commands(const string& raw) {
        string res = "";
        int n = raw.length();
        int i = 0;

        while (i < n) {
            int best_saving = -1;
            string best_replacement = "";
            int best_consumed = 1; 

            for (int len = 1; len <= 20; ++len) {
                if (i + len > n) break;
                string pat = raw.substr(i, len);
                
                int repeats = 0;
                int pos = i;
                while (pos + len <= n && raw.substr(pos, len) == pat) {
                    repeats++;
                    pos += len;
                }
                
                int consumed = repeats * len; 
                
                // 候補1: フラット展開
                string flat_str = "M" + pat + "M";
                for (int k = 1; k < repeats; ++k) flat_str += "P";
                
                // 候補2: 自己増殖展開
                string boot_str = "";
                if (repeats >= 2) {
                    boot_str = "M" + pat + pat + "M"; 
                    int executed = 2;
                    int macro_size = 2; 
                    
                    while (executed + macro_size * 2 <= repeats) {
                        boot_str += "MPPM";
                        executed += macro_size * 2;
                        macro_size *= 2;
                    }
                    
                    int rem = repeats - executed;
                    while (rem > 0) {
                        if (rem >= macro_size) {
                            boot_str += "P";
                            rem -= macro_size;
                            executed += macro_size;
                        } else {
                            boot_str += pat;
                            rem--;
                            executed++;
                        }
                    }
                }
                
                // 【修正箇所】圧縮なしの場合は「pat を repeats 回繰り返した本来の文字列」をセットする
                string candidate = ""; 
                for (int k = 0; k < repeats; ++k) {
                    candidate += pat;
                }
                
                if (repeats > 1) {
                    if (flat_str.length() < candidate.length()) {
                        candidate = flat_str;
                    }
                    if (!boot_str.empty() && boot_str.length() < candidate.length()) {
                        candidate = boot_str;
                    }
                }
                
                int saving = consumed - candidate.length();
                
                if (saving > best_saving) {
                    best_saving = saving;
                    best_replacement = candidate;
                    best_consumed = consumed;
                } else if (saving == best_saving && consumed > best_consumed) {
                    best_replacement = candidate;
                    best_consumed = consumed;
                }
            }

            if (best_saving > 0) {
                res += best_replacement;
                i += best_consumed;
            } else {
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