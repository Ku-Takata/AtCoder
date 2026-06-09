#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <algorithm>

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

// 現在の(r, c, d)からターゲットのマスへの最短操作列をBFSで求め、状態を更新する
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

        // F (前進)
        if (can_move(u.r, u.c, u.d)) {
            int nr = u.r + dr[u.d];
            int nc = u.c + dc[u.d];
            if (dist[nr][nc][u.d] > dist[u.r][u.c][u.d] + 1) {
                dist[nr][nc][u.d] = dist[u.r][u.c][u.d] + 1;
                prev[nr][nc][u.d] = u;
                move_char[nr][nc][u.d] = 'F';
                q.push({nr, nc, u.d});
            }
        }
        // R (右折)
        int nd = (u.d + 1) % 4;
        if (dist[u.r][u.c][nd] > dist[u.r][u.c][u.d] + 1) {
            dist[u.r][u.c][nd] = dist[u.r][u.c][u.d] + 1;
            prev[u.r][u.c][nd] = u;
            move_char[u.r][u.c][nd] = 'R';
            q.push({u.r, u.c, nd});
        }
        // L (左折)
        nd = (u.d + 3) % 4;
        if (dist[u.r][u.c][nd] > dist[u.r][u.c][u.d] + 1) {
            dist[u.r][u.c][nd] = dist[u.r][u.c][u.d] + 1;
            prev[u.r][u.c][nd] = u;
            move_char[u.r][u.c][nd] = 'L';
            q.push({u.r, u.c, nd});
        }
    }

    // ターゲットのマスの中で最も早く到達できる向きを探す
    int best_d = 0;
    int min_dist = 1e9;
    for (int i = 0; i < 4; ++i) {
        if (dist[target_r][target_c][i] < min_dist) {
            min_dist = dist[target_r][target_c][i];
            best_d = i;
        }
    }

    // 操作列の復元
    string path = "";
    State curr = {target_r, target_c, best_d};
    while (curr.r != curr_r || curr.c != curr_c || curr.d != curr_d) {
        path += move_char[curr.r][curr.c][curr.d];
        curr = prev[curr.r][curr.c][curr.d];
    }
    reverse(path.begin(), path.end());

    // 現在の状態を更新
    curr_r = target_r;
    curr_c = target_c;
    curr_d = best_d;

    return path;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    cin >> N >> M >> T;
    
    v_walls.resize(N);
    for (int i = 0; i < N; ++i) {
        cin >> v_walls[i];
    }
    
    h_walls.resize(N - 1);
    for (int i = 0; i < N - 1; ++i) {
        cin >> h_walls[i];
    }

    vector<Point> balls(M), baskets(M);
    for (int i = 0; i < M; ++i) {
        cin >> balls[i].r >> balls[i].c >> baskets[i].r >> baskets[i].c;
    }

    int curr_r = 0, curr_c = 0, curr_d = 0; // 初期状態: (0,0) 右向き
    vector<char> ans;

    for (int i = 0; i < M; ++i) {
        // ボールの位置へ移動
        string p1 = get_path(curr_r, curr_c, curr_d, balls[i].r, balls[i].c);
        for (char c : p1) ans.push_back(c);
        ans.push_back('S'); // 拾う

        // かごの位置へ移動
        string p2 = get_path(curr_r, curr_c, curr_d, baskets[i].r, baskets[i].c);
        for (char c : p2) ans.push_back(c);
        ans.push_back('S'); // 置く
    }

    // 出力長が T を超えないように制限
    int output_count = 0;
    for (char c : ans) {
        if (output_count >= T) break;
        cout << c << "\n";
        output_count++;
    }

    return 0;
}