#include <iostream>
#include <vector>
#include <queue>
#include <algorithm>

using namespace std;

int N = 20;
vector<vector<int>> grid(20, vector<int>(20));
vector<pair<int, int>> pos(400); // 各箱(0~399)の現在位置
vector<vector<pair<int, int>>> belts(20);
vector<pair<int, int>> ops; // 操作の記録 {ベルト番号, 方向}
int current_target = 0; // 次に搬出したい箱の番号

// --- BFS/評価用のデータ構造 ---
vector<pair<int, int>> cell_belts[20][20]; // 各マスが属する {ベルト番号, インデックス} のリスト
int dist_to_exit[20][20];                  // (0, 10) までの最短距離

// ビームサーチで状態を管理するための構造体
struct State {
    vector<vector<int>> state_grid;
    vector<pair<int, int>> state_pos;
    int state_target;
    int first_m; // この状態に至るための最初の一手（ベルト番号）
    int first_d; // この状態に至るための最初の一手（方向）
    long long score;

    // スコアが小さい（＝より出口に近い、または搬出済み）順にソートする
    bool operator<(const State& other) const {
        return score < other.score;
    }
};

void init_belts() {
    for (int i = 0; i < 10; ++i) {
        for (int c = 0; c < 20; ++c) belts[i].push_back({2 * i, c});
        for (int c = 19; c >= 0; --c) belts[i].push_back({2 * i + 1, c});
    }
    for (int j = 0; j < 10; ++j) {
        for (int r = 0; r < 20; ++r) belts[10 + j].push_back({r, 2 * j + 1});
        for (int r = 19; r >= 0; --r) belts[10 + j].push_back({r, 2 * j});
    }
}

void init_bfs() {
    for (int r = 0; r < 20; ++r) {
        for (int c = 0; c < 20; ++c) dist_to_exit[r][c] = 1e9;
    }
    
    for (int m = 0; m < 20; ++m) {
        for (int i = 0; i < 40; ++i) {
            int r = belts[m][i].first;
            int c = belts[m][i].second;
            cell_belts[r][c].push_back({m, i});
        }
    }

    queue<pair<int, int>> q;
    q.push({0, 10});
    dist_to_exit[0][10] = 0;

    while (!q.empty()) {
        auto p = q.front();
        int r = p.first;
        int c = p.second;
        q.pop();

        for (auto b : cell_belts[r][c]) {
            int m = b.first;
            int idx = b.second;

            int prev_idx1 = (idx - 1 + 40) % 40;
            int pr1 = belts[m][prev_idx1].first;
            int pc1 = belts[m][prev_idx1].second;
            if (dist_to_exit[pr1][pc1] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr1][pc1] = dist_to_exit[r][c] + 1;
                q.push({pr1, pc1});
            }

            int prev_idx2 = (idx + 1) % 40;
            int pr2 = belts[m][prev_idx2].first;
            int pc2 = belts[m][prev_idx2].second;
            if (dist_to_exit[pr2][pc2] > dist_to_exit[r][c] + 1) {
                dist_to_exit[pr2][pc2] = dist_to_exit[r][c] + 1;
                q.push({pr2, pc2});
            }
        }
    }
}

// 評価関数
long long evaluate(const vector<pair<int, int>>& current_pos, int base_target, int state_target) {
    long long total_score = 0;
    
    // シミュレーション中に箱が搬出された場合、莫大なボーナス（マイナススコア）を与える
    long long out_bonus = -1000000000000000LL; 
    total_score += (state_target - base_target) * out_bonus;

    long long weights[] = {1000000000000LL, 100000000LL, 10000LL, 1LL}; 
    int num_lookahead = 4;

    for (int offset = 0; offset < num_lookahead; ++offset) {
        int check_id = state_target + offset;
        if (check_id >= 400) break;
        if (current_pos[check_id].first == -1) continue;
        int r = current_pos[check_id].first;
        int c = current_pos[check_id].second;
        total_score += weights[offset] * dist_to_exit[r][c];
    }
    return total_score;
}

// ビームサーチで「次の一手」を決定する
pair<int, int> get_best_move_beam_search(int base_target) {
    // 探索の幅と深さ。TLEになる場合はここを小さく、余裕があれば大きくする
    int BEAM_WIDTH = 1000000;
    int BEAM_DEPTH = 1;
    
    vector<State> beam;
    State initial_state;
    initial_state.state_grid = grid;
    initial_state.state_pos = pos;
    initial_state.state_target = base_target;
    initial_state.first_m = -1;
    initial_state.first_d = 0;
    initial_state.score = evaluate(pos, base_target, base_target);
    beam.push_back(initial_state);
    
    for (int depth = 0; depth < BEAM_DEPTH; ++depth) {
        vector<State> next_beam;
        
        for (auto& st : beam) {
            // 計算量を抑えるため、現在のターゲット周辺のベルトのみを候補にする
            vector<int> candidate_belts;
            for (int offset = 0; offset < 4; ++offset) {
                int check_id = st.state_target + offset;
                if (check_id >= 400) break;
                if (st.state_pos[check_id].first == -1) continue;
                int r = st.state_pos[check_id].first;
                int c = st.state_pos[check_id].second;
                for (auto b : cell_belts[r][c]) {
                    candidate_belts.push_back(b.first);
                }
            }
            // 重複を排除
            sort(candidate_belts.begin(), candidate_belts.end());
            candidate_belts.erase(unique(candidate_belts.begin(), candidate_belts.end()), candidate_belts.end());

            for (int m : candidate_belts) {
                for (int d : {1, -1}) {
                    State next_st = st; // 状態のコピー
                    
                    // シミュレーション（1手進める）
                    vector<int> temp(40);
                    for (int i = 0; i < 40; ++i) {
                        temp[i] = next_st.state_grid[belts[m][i].first][belts[m][i].second];
                    }
                    for (int i = 0; i < 40; ++i) {
                        int new_idx = (i + d + 40) % 40;
                        int box_id = temp[i];
                        next_st.state_grid[belts[m][new_idx].first][belts[m][new_idx].second] = box_id;
                        if (box_id != -1) {
                            next_st.state_pos[box_id] = belts[m][new_idx];
                        }
                    }
                    
                    // 搬出判定
                    if (next_st.state_grid[0][10] == next_st.state_target) {
                        next_st.state_grid[0][10] = -1;
                        next_st.state_pos[next_st.state_target] = {-1, -1};
                        next_st.state_target++;
                    }

                    // 評価
                    next_st.score = evaluate(next_st.state_pos, base_target, next_st.state_target);
                    
                    // 深さ0の時、この状態に至るための「最初の一手」を記録
                    if (depth == 0) {
                        next_st.first_m = m;
                        next_st.first_d = d;
                    }
                    next_beam.push_back(next_st);
                }
            }
        }
        
        // 評価値が良い順にソートし、上位 BEAM_WIDTH 個を残す
        sort(next_beam.begin(), next_beam.end());
        if (next_beam.size() > BEAM_WIDTH) {
            next_beam.resize(BEAM_WIDTH);
        }
        beam = next_beam;
    }
    
    // 探索終了後、最も評価が高かった状態の「最初の一手」を返す
    return {beam[0].first_m, beam[0].first_d};
}

void shift_belt(int m, int d) {
    vector<int> temp(40);
    for (int i = 0; i < 40; ++i) {
        temp[i] = grid[belts[m][i].first][belts[m][i].second];
    }
    for (int i = 0; i < 40; ++i) {
        int new_idx = (i + d + 40) % 40;
        int box_id = temp[i];
        grid[belts[m][new_idx].first][belts[m][new_idx].second] = box_id;
        if (box_id != -1) {
            pos[box_id] = belts[m][new_idx];
        }
    }
    ops.push_back({m, d});

    if (grid[0][10] == current_target) {
        grid[0][10] = -1;
        pos[current_target] = {-1, -1};
        current_target++;
    }
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    int dummy_N;
    if (!(cin >> dummy_N)) return 0;

    for (int i = 0; i < 20; ++i) {
        for (int j = 0; j < 20; ++j) {
            cin >> grid[i][j];
            pos[grid[i][j]] = {i, j};
        }
    }

    init_belts();
    init_bfs();

    if (grid[0][10] == 0) {
        grid[0][10] = -1;
        pos[0] = {-1, -1};
        current_target = 1;
    }

    while (current_target < 400) {
        if (pos[current_target].first == -1) {
            current_target++;
            continue;
        }

        if (pos[current_target] == make_pair(0, 10)) {
            shift_belt(1, 1);
            shift_belt(1, -1);
            continue;
        }

        int target_id = current_target;

        while (current_target == target_id && pos[target_id] != make_pair(0, 10)) {
            // ビームサーチを呼び出して次の一手を決める
            pair<int, int> best_move = get_best_move_beam_search(current_target);
            shift_belt(best_move.first, best_move.second);
        }
    }

    cout << 20 << "\n";
    for (int i = 0; i < 20; ++i) {
        cout << 40;
        for (auto p : belts[i]) {
            cout << " " << p.first << " " << p.second;
        }
        cout << "\n";
    }
    
    cout << ops.size() << "\n";
    for (auto op : ops) {
        cout << op.first << " " << op.second << "\n";
    }

    return 0;
}