#include <iostream>
#include <vector>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <algorithm>
#include <cstdint>

using namespace std;

const int dr[] = {0, 1, 0, -1}; // 0:右, 1:下, 2:左, 3:上
const int dc[] = {1, 0, -1, 0};
const int MAX_MACRO_LEN = 6;    // スモールケース向けのマクロ長上限

int N, M, T;
vector<vector<bool>> wall_v;
vector<vector<bool>> wall_h;
vector<int> basket_pos;

// マクロ文字列を整数IDで管理し、メモリを極限まで節約する
vector<string> id_to_macro;
unordered_map<string, int> macro_to_id;

int get_macro_id(const string& str) {
    if (str.empty()) return 0;
    auto it = macro_to_id.find(str);
    if (it != macro_to_id.end()) return it->second;
    int new_id = id_to_macro.size();
    id_to_macro.push_back(str);
    macro_to_id[str] = new_id;
    return new_id;
}

// 16バイトに圧縮した状態構造体
#pragma pack(push, 1)
struct State {
    int8_t r, c, dir, holding;
    int8_t is_rec;
    int reg_id, rec_id;
    int8_t ball_pos[6]; // 制約外スモールケース(M<=6)を想定

    bool operator==(const State& o) const {
        if (r != o.r || c != o.c || dir != o.dir || holding != o.holding || 
            is_rec != o.is_rec || reg_id != o.reg_id || rec_id != o.rec_id) return false;
        for(int i = 0; i < M; ++i) if(ball_pos[i] != o.ball_pos[i]) return false;
        return true;
    }
};
#pragma pack(pop)

// 状態のハッシュ関数
struct StateHash {
    size_t operator()(const State& s) const {
        size_t res = 17;
        res = res * 31 + s.r;       res = res * 31 + s.c;
        res = res * 31 + s.dir;     res = res * 31 + s.holding;
        res = res * 31 + s.is_rec;
        res = res * 31 + s.reg_id;  res = res * 31 + s.rec_id;
        for (int i = 0; i < M; ++i) res = res * 31 + s.ball_pos[i];
        return res;
    }
};

// BFS木ノード
struct Node {
    State s;
    int parent_idx;
    char action;
    uint16_t expanded_ops; // 展開後の基本操作数 (T以下である必要がある)
};

bool is_goal(const State& s) {
    for(int i = 0; i < M; ++i) {
        if (s.ball_pos[i] != basket_pos[i]) return false;
    }
    return true;
}

// 基本操作の適用 (is_manual=trueなら、壁への衝突などの無駄な操作時にfalseを返し枝刈りする)
bool apply_base_op(State& s, char op, bool is_manual) {
    if (op == 'F') {
        int nr = s.r + dr[s.dir];
        int nc = s.c + dc[s.dir];
        bool can_move = true;
        if (nr < 0 || nr >= N || nc < 0 || nc >= N) can_move = false;
        else {
            if (s.dir == 0 && wall_v[s.r][s.c]) can_move = false;
            if (s.dir == 1 && wall_h[s.r][s.c]) can_move = false;
            if (s.dir == 2 && wall_v[s.r][s.c - 1]) can_move = false;
            if (s.dir == 3 && wall_h[s.r - 1][s.c]) can_move = false;
        }
        if (can_move) { s.r = nr; s.c = nc; return true; }
        return !is_manual; // マクロ内の衝突は許容するが、手動での衝突は枝刈り
    } else if (op == 'R') {
        s.dir = (s.dir + 1) % 4; return true;
    } else if (op == 'L') {
        s.dir = (s.dir + 3) % 4; return true;
    } else if (op == 'S') {
        int pos = s.r * N + s.c;
        int on_ground = -1;
        for (int i = 0; i < M; ++i) {
            if (s.ball_pos[i] == pos) { on_ground = i; break; }
        }
        if (s.holding == -1 && on_ground == -1) return !is_manual; // 空振りは枝刈り
        
        if (on_ground != -1) s.ball_pos[on_ground] = N * N; // 持っている状態を N*N で表現
        if (s.holding != -1) s.ball_pos[s.holding] = pos;
        s.holding = on_ground;
        return true;
    }
    return false;
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    if (!(cin >> N >> M >> T)) return 0;
    
    // 辞書の初期化 (ID:0 は空文字)
    id_to_macro.push_back("");
    macro_to_id[""] = 0;

    wall_v.assign(N, vector<bool>(N - 1, false));
    wall_h.assign(N - 1, vector<bool>(N, false));

    for (int i = 0; i < N; ++i) {
        string s; cin >> s;
        for (int j = 0; j < N - 1; ++j) wall_v[i][j] = (s[j] == '1');
    }
    for (int i = 0; i < N - 1; ++i) {
        string s; cin >> s;
        for (int j = 0; j < N; ++j) wall_h[i][j] = (s[j] == '1');
    }

    State initial_state = {0, 0, 0, -1, 0, 0, 0, {0}};
    basket_pos.resize(M);

    for (int k = 0; k < M; ++k) {
        int br, bc, dr_b, dc_b;
        cin >> br >> bc >> dr_b >> dc_b;
        initial_state.ball_pos[k] = br * N + bc;
        basket_pos[k] = dr_b * N + dc_b;
    }

    vector<Node> nodes;
    nodes.reserve(30000000); // 3000万状態までメモリ確保
    unordered_set<State, StateHash> visited;
    visited.reserve(30000000);

    nodes.push_back({initial_state, -1, ' ', 0});
    visited.insert(initial_state);

    int head = 0;
    while (head < nodes.size()) {
        State curr = nodes[head].s;
        char prev_op = nodes[head].action;
        uint16_t exp_ops = nodes[head].expanded_ops;

        // ゴール到達！ (BFSなので到達時点でのボタン操作回数が最小になる)
        if (is_goal(curr)) {
            string result = "";
            int curr_idx = head;
            while (nodes[curr_idx].parent_idx != -1) {
                result += nodes[curr_idx].action;
                curr_idx = nodes[curr_idx].parent_idx;
            }
            reverse(result.begin(), result.end());
            
            cerr << "[DEBUG] ゴール到達！ 文字列長: " << result.length() 
                 << ", 展開後操作数: " << exp_ops << "\n";
            for (char c : result) cout << c << "\n";
            return 0;
        }

        auto try_push = [&](const State& ns, char op, uint16_t next_exp) {
            if (next_exp <= T && visited.find(ns) == visited.end()) {
                visited.insert(ns);
                nodes.push_back({ns, head, op, next_exp});
            }
        };

        // 進行状況の表示
        if (head % 500000 == 0 && head > 0) {
            cerr << "[DEBUG] 探索状態数: " << head << "\n";
        }

        // 1. 基本操作 (F, R, L, S)
        char base_ops[] = {'F', 'R', 'L', 'S'};
        for (char op : base_ops) {
            // 枝刈り：無意味なターンを禁止
            if (op == 'L' && prev_op == 'R') continue;
            if (op == 'R' && prev_op == 'L') continue;
            
            State ns = curr;
            if (!apply_base_op(ns, op, true)) continue;

            if (ns.is_rec) {
                string next_rec = id_to_macro[ns.rec_id] + op;
                if (next_rec.length() > MAX_MACRO_LEN) continue;
                ns.rec_id = get_macro_id(next_rec);
            }
            try_push(ns, op, exp_ops + 1);
        }

        // 2. マクロ操作 (M)
        if (prev_op != 'M') { // 枝刈り：Mの連続を防ぐ
            State ns_m = curr;
            if (!ns_m.is_rec) {
                ns_m.is_rec = true;
                ns_m.rec_id = 0;
            } else {
                ns_m.is_rec = false;
                ns_m.reg_id = ns_m.rec_id;
            }
            try_push(ns_m, 'M', exp_ops);
        }

        // 3. 再生操作 (P)
        if (curr.reg_id != 0) {
            State ns_p = curr;
            bool p_valid = true;
            string macro_str = id_to_macro[curr.reg_id];
            
            for (char c : macro_str) {
                apply_base_op(ns_p, c, false);
                if (ns_p.is_rec) {
                    string next_rec = id_to_macro[ns_p.rec_id] + c;
                    if (next_rec.length() > MAX_MACRO_LEN) { p_valid = false; break; }
                    ns_p.rec_id = get_macro_id(next_rec);
                }
            }
            if (p_valid) {
                try_push(ns_p, 'P', exp_ops + macro_str.length());
            }
        }
        
        head++;
    }

    cerr << "[DEBUG] 探索終了: 答えが見つかりませんでした (Tの制限超過、または到達不能)\n";
    return 0;
}