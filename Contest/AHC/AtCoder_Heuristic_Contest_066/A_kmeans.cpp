#include <iostream>
#include <vector>
#include <string>
#include <queue>
#include <cmath>
#include <map>
#include <algorithm>
#include <random>

using namespace std;

// 方向の定義: 0=右, 1=下, 2=左, 3=上
const int dr[] = {0, 1, 0, -1};
const int dc[] = {1, 0, -1, 0};

struct Point {
    int r, c;
    bool operator==(const Point& o) const { return r == o.r && c == o.c; }
    bool operator!=(const Point& o) const { return !(*this == o); }
};

struct Ball {
    int id;
    Point start;
    Point goal;
    int cluster_id;
};

int N, M_balls, T;
vector<string> v; // 縦の壁
vector<string> h; // 横の壁
vector<Ball> balls;

// 壁の判定
bool can_move(int r, int c, int dir) {
    if (dir == 0) { // 右
        if (c + 1 >= N || v[r][c] == '1') return false;
    } else if (dir == 1) { // 下
        if (r + 1 >= N || h[r][c] == '1') return false;
    } else if (dir == 2) { // 左
        if (c - 1 < 0 || v[r][c - 1] == '1') return false;
    } else if (dir == 3) { // 上
        if (r - 1 < 0 || h[r - 1][c] == '1') return false;
    }
    return true;
}

// 状態遷移のための構造体
struct State {
    int r, c, dir;
};

// 2状態間の最短経路(基本操作 F, R, L の列)を求めるBFS
string get_path(int sr, int sc, int sdir, int gr, int gc) {
    if (sr == gr && sc == gc) return "";

    int dist[25][25][4];
    int prev_action[25][25][4]; // 0:F, 1:R, 2:L
    State prev_state[25][25][4];

    for(int i=0; i<N; ++i) for(int j=0; j<N; ++j) for(int k=0; k<4; ++k) dist[i][j][k] = 1e9;
    
    queue<State> q;
    q.push({sr, sc, sdir});
    dist[sr][sc][sdir] = 0;

    bool found = false;
    int end_dir = -1;

    while (!q.empty()) {
        State cur = q.front();
        q.pop();

        if (cur.r == gr && cur.c == gc) {
            found = true;
            end_dir = cur.dir;
            break;
        }

        // 1. 前進 (F)
        if (can_move(cur.r, cur.c, cur.dir)) {
            int nr = cur.r + dr[cur.dir];
            int nc = cur.c + dc[cur.dir];
            if (dist[nr][nc][cur.dir] > dist[cur.r][cur.c][cur.dir] + 1) {
                dist[nr][nc][cur.dir] = dist[cur.r][cur.c][cur.dir] + 1;
                prev_action[nr][nc][cur.dir] = 0;
                prev_state[nr][nc][cur.dir] = cur;
                q.push({nr, nc, cur.dir});
            }
        }
        // 2. 右折 (R)
        int ndir_r = (cur.dir + 1) % 4;
        if (dist[cur.r][cur.c][ndir_r] > dist[cur.r][cur.c][cur.dir] + 1) {
            dist[cur.r][cur.c][ndir_r] = dist[cur.r][cur.c][cur.dir] + 1;
            prev_action[cur.r][cur.c][ndir_r] = 1;
            prev_state[cur.r][cur.c][ndir_r] = cur;
            q.push({cur.r, cur.c, ndir_r});
        }
        // 3. 左折 (L)
        int ndir_l = (cur.dir + 3) % 4;
        if (dist[cur.r][cur.c][ndir_l] > dist[cur.r][cur.c][cur.dir] + 1) {
            dist[cur.r][cur.c][ndir_l] = dist[cur.r][cur.c][cur.dir] + 1;
            prev_action[cur.r][cur.c][ndir_l] = 2;
            prev_state[cur.r][cur.c][ndir_l] = cur;
            q.push({cur.r, cur.c, ndir_l});
        }
    }

    if (!found) return "";

    string path = "";
    State curr = {gr, gc, end_dir};
    while (!(curr.r == sr && curr.c == sc && curr.dir == sdir)) {
        int act = prev_action[curr.r][curr.c][curr.dir];
        if (act == 0) path += "F";
        else if (act == 1) path += "R";
        else if (act == 2) path += "L";
        curr = prev_state[curr.r][curr.c][curr.dir];
    }
    reverse(path.begin(), path.end());
    return path;
}

// 距離計算（マンハッタン距離）
int manhattan(Point a, Point b) {
    return abs(a.r - b.r) + abs(a.c - b.c);
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    if (!(cin >> N >> M_balls >> T)) return 0;

    v.resize(N);
    for (int i = 0; i < N; ++i) cin >> v[i];
    h.resize(N - 1);
    for (int i = 0; i < N - 1; ++i) cin >> h[i];

    balls.resize(M_balls);
    for (int i = 0; i < M_balls; ++i) {
        cin >> balls[i].start.r >> balls[i].start.c >> balls[i].goal.r >> balls[i].goal.c;
        balls[i].id = i;
    }

    cerr << "[DEBUG] Board Size: " << N << "x" << N << ", Balls: " << M_balls << ", Max T: " << T << endl;

    // --- 1. K-means によるクラスタリング ---
    int K = max(1, M_balls / 4); // クラスタ数の初期値（要調整パラメータ）
    cerr << "[DEBUG] Target Clusters (K): " << K << endl;

    vector<Point> centroids(K);
    mt19937 mt(42);
    for (int i = 0; i < K; ++i) {
        centroids[i] = balls[mt() % M_balls].goal; // かごの位置を基準に初期化
    }

    for (int iter = 0; iter < 10; ++iter) {
        vector<Point> new_centroids(K, {0, 0});
        vector<int> counts(K, 0);

        for (auto& b : balls) {
            int best_c = 0, min_dist = 1e9;
            for (int i = 0; i < K; ++i) {
                int d = manhattan(b.goal, centroids[i]); // かごの近さでグルーピング
                if (d < min_dist) { min_dist = d; best_c = i; }
            }
            b.cluster_id = best_c;
            new_centroids[best_c].r += b.goal.r;
            new_centroids[best_c].c += b.goal.c;
            counts[best_c]++;
        }

        for (int i = 0; i < K; ++i) {
            if (counts[i] > 0) {
                centroids[i].r = new_centroids[i].r / counts[i];
                centroids[i].c = new_centroids[i].c / counts[i];
            }
        }
    }

    // --- 2. 各クラスタの集約地点（直線のマス）の探索 ---
    // ここでは単純化のため、各重心の近くで横方向に連続して空いているマス群を探す
    map<int, vector<Point>> cluster_assembly_lines;
    for (int k = 0; k < K; ++k) {
        int target_r = centroids[k].r;
        vector<Point> line;
        // 重心行で壁に阻まれない連続区間を取得
        for (int c = 0; c < N; ++c) {
            line.push_back({target_r, c});
            if (c + 1 < N && v[target_r][c] == '1') {
                if (line.size() >= 2) break; // ある程度の長さがあれば採用
                line.clear();
            }
        }
        cluster_assembly_lines[k] = line;
        cerr << "[DEBUG] Cluster " << k << " assembly line row: " << target_r << ", length: " << line.size() << endl;
    }

    // --- 3. 実際の操作生成 ---
    // ロボットの初期状態
    int cur_r = 0, cur_c = 0, cur_dir = 0;
    string total_actions = "";

    auto move_to = [&](int gr, int gc) {
        string path = get_path(cur_r, cur_c, cur_dir, gr, gc);
        total_actions += path;
        cur_r = gr; cur_c = gc;
        if (!path.empty()) {
            for (char p : path) {
                if (p == 'R') cur_dir = (cur_dir + 1) % 4;
                if (p == 'L') cur_dir = (cur_dir + 3) % 4;
            }
        }
    };

    // フェーズ1: ボールをピックアップして集約地点に運ぶ（現状はベースとして愚直に処理）
    // マクロを構築しやすくするために「集める」
    for (int k = 0; k < K; ++k) {
        auto& line = cluster_assembly_lines[k];
        if (line.empty()) continue;
        int line_idx = 0;

        for (auto& b : balls) {
            if (b.cluster_id == k) {
                // ボールの位置へ移動
                move_to(b.start.r, b.start.c);
                total_actions += "S"; // 拾う
                
                // 集約地点へ移動
                Point target_pt = line[line_idx % line.size()];
                move_to(target_pt.r, target_pt.c);
                total_actions += "S"; // 置く
                b.start = target_pt; // 現在地を更新
                line_idx++;
                
                cerr << "[DEBUG] Ball " << b.id << " moved to assembly line (" << target_pt.r << "," << target_pt.c << ")" << endl;
            }
        }
    }

    // フェーズ2: 集約地点からかごへ運ぶ（ここでマクロを活用する余地を作る）
    // TODO: ここで、共通の移動パターン（例: FFFRFSなど）を抽出し、Mで記録、Pで再生するロジックを組み込む
    // 今回はプレースホルダーとして、集約後の地点からかごへ愚直に運ぶロジックとしています。
    cerr << "[DEBUG] Start Phase 2: Assembly -> Goal" << endl;
    for (auto& b : balls) {
        move_to(b.start.r, b.start.c);
        total_actions += "S"; // 拾う
        
        // かごへ移動
        move_to(b.goal.r, b.goal.c);
        total_actions += "S"; // 置く
    }

    // --- 4. 結果の出力 ---
    cerr << "[DEBUG] Total Actions Length: " << total_actions.size() << " / " << T << endl;
    for (int i = 0; i < min((int)total_actions.size(), T); ++i) {
        cout << total_actions[i] << "\n";
    }

    return 0;
}