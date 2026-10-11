use std::io::{self, Read, Write};
use std::time::Instant;

const DIR: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const DIR_CHAR: [char; 4] = ['U', 'D', 'L', 'R'];

#[derive(Clone, Copy, Debug)]
struct Op {
    r: usize,
    c: usize,
    k: usize,
    d: char,
    l: usize,
}

struct XorShift {
    state: u64,
}

impl XorShift {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }
    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    #[inline(always)]
    fn gen_range(&mut self, l: usize, r: usize) -> usize {
        l + (self.next_u64() as usize % (r - l))
    }
    #[inline(always)]
    fn gen_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

// ビットパッキング盤面（サイズ 2.0KB）
#[derive(Clone, Copy, PartialEq, Eq)]
struct BitBoard {
    colors: [u32; 400],
    lens: [u8; 400],
}

impl BitBoard {
    fn new(n: usize, init_stacks: &[Vec<usize>]) -> Self {
        let mut board = Self {
            colors: [0; 400],
            lens: [0; 400],
        };
        for p in 0..n * n {
            let len = init_stacks[p].len();
            board.lens[p] = len as u8;
            let mut c_bits = 0u32;
            for i in 0..len {
                c_bits |= ((init_stacks[p][i] & 0xF) as u32) << (i * 4);
            }
            board.colors[p] = c_bits;
        }
        board
    }

    #[inline(always)]
    fn len(&self, p: usize) -> usize {
        self.lens[p] as usize
    }

    #[inline(always)]
    fn is_empty(&self, p: usize) -> bool {
        self.lens[p] == 0
    }

    #[inline(always)]
    fn last(&self, p: usize) -> Option<usize> {
        let l = self.lens[p] as usize;
        if l == 0 {
            None
        } else {
            let shift = (l - 1) * 4;
            Some(((self.colors[p] >> shift) & 0xF) as usize)
        }
    }

    #[inline(always)]
    fn contains(&self, p: usize, color: usize) -> bool {
        let l = self.lens[p] as usize;
        let mut c = self.colors[p];
        for _ in 0..l {
            if ((c & 0xF) as usize) == color {
                return true;
            }
            c >>= 4;
        }
        false
    }

    #[inline(always)]
    fn transfer_and_reverse(&mut self, p: usize, next_p: usize, k: usize) {
        let h = self.lens[p] as usize;
        let block_len = h - k;
        let shift_k = (k * 4) as u32;
        let mask = if block_len >= 8 {
            0xFFFF_FFFF
        } else {
            (1u32 << (block_len * 4)) - 1
        };
        let block = (self.colors[p] >> shift_k) & mask;

        self.lens[p] = k as u8;
        self.colors[p] &= if k == 0 {
            0
        } else {
            (1u32 << shift_k) - 1
        };

        let mut rev = 0u32;
        let mut temp = block;
        for _ in 0..block_len {
            rev = (rev << 4) | (temp & 0xF);
            temp >>= 4;
        }

        let next_l = self.lens[next_p] as usize;
        let shift_next = (next_l * 4) as u32;
        self.colors[next_p] |= rev << shift_next;
        self.lens[next_p] += block_len as u8;
    }

    #[inline(always)]
    fn clean_nest(&mut self, p: usize, plate_color: usize) -> usize {
        let mut cleaned = 0;
        while self.lens[p] > 0 {
            let shift = (self.lens[p] as usize - 1) * 4;
            if ((self.colors[p] >> shift) & 0xF) as usize == plate_color {
                self.lens[p] -= 1;
                let new_l = self.lens[p] as usize;
                self.colors[p] &= if new_l == 0 { 0 } else { (1u32 << (new_l * 4)) - 1 };
                cleaned += 1;
            } else {
                break;
            }
        }
        cleaned
    }

    #[inline(always)]
    fn count_color(&self, color: usize) -> usize {
        let mut count = 0;
        for p in 0..400 {
            let l = self.lens[p] as usize;
            if l == 0 { continue; }
            let mut c = self.colors[p];
            for _ in 0..l {
                if ((c & 0xF) as usize) == color {
                    count += 1;
                }
                c >>= 4;
            }
        }
        count
    }
}

struct BfsContext {
    token: u32,
    visited: [u32; 400],
    dist: [u32; 400],
    prev: [usize; 400],
    prev_action: [(usize, usize, usize); 400],
    queue: [usize; 400],
    actions: [(usize, usize, usize, usize); 400],

    token_tower: u32,
    visited_tower: [u32; 3600],
    dist_tower: [u32; 3600],
    prev_tower: [(usize, usize); 3600],
    queue_tower: [usize; 3600],
    path_tower: [usize; 400],
}

impl BfsContext {
    fn new() -> Self {
        Self {
            token: 0,
            visited: [0; 400],
            dist: [0; 400],
            prev: [0; 400],
            prev_action: [(0, 0, 0); 400],
            queue: [0; 400],
            actions: [(0, 0, 0, 0); 400],

            token_tower: 0,
            visited_tower: [0; 3600],
            dist_tower: [0; 3600],
            prev_tower: [(0, 0); 3600],
            queue_tower: [0; 3600],
            path_tower: [0; 400],
        }
    }
}

fn compute_max_ray(n: usize, floor: &[bool]) -> [[usize; 4]; 400] {
    let mut max_ray = [[0usize; 4]; 400];
    for r in 0..n {
        for c in 0..n {
            let u = r * n + c;
            if !floor[u] {
                continue;
            }
            let mut l = 0;
            while r >= l + 1 && floor[(r - 1 - l) * n + c] {
                l += 1;
            }
            max_ray[u][0] = l;

            let mut l = 0;
            while r + 1 + l < n && floor[(r + 1 + l) * n + c] {
                l += 1;
            }
            max_ray[u][1] = l;

            let mut l = 0;
            while c >= l + 1 && floor[r * n + (c - 1 - l)] {
                l += 1;
            }
            max_ray[u][2] = l;

            let mut l = 0;
            while c + 1 + l < n && floor[r * n + (c + 1 + l)] {
                l += 1;
            }
            max_ray[u][3] = l;
        }
    }
    max_ray
}

fn compute_all_pairs_dist(n: usize, floor: &[bool]) -> Vec<Vec<usize>> {
    let mut dist = vec![vec![usize::MAX; n * n]; n * n];
    let mut q = [0usize; 400];
    for start in 0..n * n {
        if !floor[start] {
            continue;
        }
        dist[start][start] = 0;
        let mut head = 0;
        let mut tail = 0;
        q[tail] = start;
        tail += 1;
        while head < tail {
            let u = q[head];
            head += 1;
            let r = u / n;
            let c = u % n;
            for d in 0..4 {
                let nr = r as isize + DIR[d].0;
                let nc = c as isize + DIR[d].1;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                    continue;
                }
                let v = nr as usize * n + nc as usize;
                if floor[v] && dist[start][v] == usize::MAX {
                    dist[start][v] = dist[start][u] + 1;
                    q[tail] = v;
                    tail += 1;
                }
            }
        }
    }
    dist
}

// -------------------------------------------------------------
// タワー合体歩行 BFS（途中マスにスライムがいても合体して <= 8 なら通過可能！）
// -------------------------------------------------------------
fn find_path_merge_fast(
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &BitBoard,
    plates: &[usize],
    start: usize,
    target: usize,
    start_h: usize,
    ctx: &mut BfsContext,
) -> Option<usize> {
    ctx.token_tower += 1;
    let token = ctx.token_tower;

    let target_color = plates.iter().position(|&pos| pos == target).or_else(|| board.last(start));

    let start_idx = start * 9 + start_h;
    ctx.visited_tower[start_idx] = token;
    ctx.dist_tower[start_idx] = 0;

    let mut head = 0;
    let mut tail = 0;
    ctx.queue_tower[tail] = start_idx;
    tail += 1;

    let mut best_target_idx = None;
    let mut best_target_dist = u32::MAX;

    while head < tail {
        let curr_idx = ctx.queue_tower[head];
        head += 1;

        let u = curr_idx / 9;
        let h = curr_idx % 9;
        let d_u = ctx.dist_tower[curr_idx];

        if u == target {
            if d_u < best_target_dist {
                best_target_dist = d_u;
                best_target_idx = Some(curr_idx);
            }
            continue;
        }

        if d_u >= best_target_dist {
            continue;
        }

        for d in 0..4 {
            if max_ray[u][d] < 1 {
                continue;
            }
            let next_p = (u as isize + dir_step[d]) as usize;

            let cur_dest_len = board.len(next_p);
            let mut new_h = cur_dest_len + h;

            if new_h > 8 {
                continue;
            }

            if plates.contains(&next_p) {
                if let Some(target_c) = target_color {
                    if let Some(pos) = plates.iter().position(|&p| p == next_p) {
                        if pos == target_c {
                            new_h = 0;
                        }
                    }
                }
            }

            let next_idx = next_p * 9 + new_h;
            if ctx.visited_tower[next_idx] != token || d_u + 1 < ctx.dist_tower[next_idx] {
                ctx.visited_tower[next_idx] = token;
                ctx.dist_tower[next_idx] = d_u + 1;
                ctx.prev_tower[next_idx] = (curr_idx, d);
                ctx.queue_tower[tail] = next_idx;
                tail += 1;
            }
        }
    }

    let end_idx = best_target_idx?;

    let mut step_count = 0;
    let mut curr = end_idx;
    while curr != start_idx && step_count < 400 {
        let (p_idx, d) = ctx.prev_tower[curr];
        ctx.path_tower[step_count] = d;
        step_count += 1;
        curr = p_idx;
    }
    ctx.path_tower[..step_count].reverse();
    Some(step_count)
}

// -------------------------------------------------------------
// タワーを目的地へ移動させる関数
// 1. 直線大ジャンプ（壁がなければ一撃）
// 2. タワー合体歩行 BFS（find_path_merge_fast）
// -------------------------------------------------------------
fn move_tower_to_target(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    start: usize,
    target: usize,
    ctx: &mut BfsContext,
) -> bool {
    if start == target || board.is_empty(start) {
        return true;
    }

    let h = board.len(start);
    let r_s = start / n;
    let c_s = start % n;
    let r_t = target / n;
    let c_t = target % n;

    // 1. 直線上の判定
    let mut on_axis = None;
    if r_s == r_t {
        if c_t > c_s {
            on_axis = Some((3, c_t - c_s));
        } else if c_s > c_t {
            on_axis = Some((2, c_s - c_t));
        }
    } else if c_s == c_t {
        if r_t > r_s {
            on_axis = Some((1, r_t - r_s));
        } else if r_s > r_t {
            on_axis = Some((0, r_s - r_t));
        }
    }

    if let Some((d, dist_l)) = on_axis {
        if max_ray[start][d] >= dist_l {
            // 隣接マス（dist_l == 1）：タワー全員一撃移動
            if dist_l == 1 && board.len(target) + h <= 8 {
                ops.push(Op {
                    r: r_s,
                    c: c_s,
                    k: 0,
                    d: DIR_CHAR[d],
                    l: 1,
                });
                board.transfer_and_reverse(start, target, 0);
                for &chk in &[start, target] {
                    if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                        board.clean_nest(chk, pc);
                    }
                }
                return true;
            }

            // 巣へのダンク射出（dist_l <= h かつ target が巣）
            if dist_l > 1 && h >= dist_l && plates.contains(&target) {
                let k = dist_l - 1;
                let flying = h - k;
                if board.len(target) + flying <= 8 {
                    ops.push(Op {
                        r: r_s,
                        c: c_s,
                        k,
                        d: DIR_CHAR[d],
                        l: dist_l,
                    });
                    board.transfer_and_reverse(start, target, k);
                    for &chk in &[start, target] {
                        if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                            board.clean_nest(chk, pc);
                        }
                    }
                    if board.is_empty(start) {
                        return true;
                    }
                }
            }
        }
    }

    // 2. 合体歩行 BFS
    if let Some(step_count) = find_path_merge_fast(dir_step, max_ray, board, plates, start, target, h, ctx) {
        let mut curr = start;
        for idx in 0..step_count {
            let d = ctx.path_tower[idx];
            let next_p = (curr as isize + dir_step[d]) as usize;

            ops.push(Op {
                r: curr / n,
                c: curr % n,
                k: 0,
                d: DIR_CHAR[d],
                l: 1,
            });

            board.transfer_and_reverse(curr, next_p, 0);

            for &p in &[curr, next_p] {
                if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                    board.clean_nest(p, plate_color);
                }
            }

            curr = next_p;
            if curr == target || board.is_empty(curr) {
                break;
            }
        }
        return true;
    }

    false
}

// -------------------------------------------------------------
// 隣接マージのO(1)即時実行
// -------------------------------------------------------------
fn try_merge_adjacent_fast(
    n: usize,
    dir_step: &[isize; 4],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    u: usize,
    v: usize,
) -> bool {
    let r_u = u / n;
    let c_u = u % n;
    let r_v = v / n;
    let c_v = v % n;

    let d = if r_u + 1 == r_v && c_u == c_v {
        1 // D
    } else if r_u == r_v + 1 && c_u == c_v {
        0 // U
    } else if r_u == r_v && c_u + 1 == c_v {
        3 // R
    } else if r_u == r_v && c_u == c_v + 1 {
        2 // L
    } else {
        return false;
    };

    let h_u = board.len(u);
    let h_v = board.len(v);
    if h_u == 0 || h_u + h_v > 8 {
        return false;
    }

    ops.push(Op {
        r: r_u,
        c: c_u,
        k: 0,
        d: DIR_CHAR[d],
        l: 1,
    });
    board.transfer_and_reverse(u, v, 0);
    for &chk in &[u, v] {
        if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
            board.clean_nest(chk, pc);
        }
    }
    true
}

// -------------------------------------------------------------
// 高度な階層的クラスタリング＆高速タワー射出ソルバー
// -------------------------------------------------------------
fn solve_hierarchical(
    n: usize,
    _k: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    _floor: &[bool],
    init_board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    color_order: &[usize],
    max_merge_dist: usize,
    target_tower_limit: usize,
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(300);

    // =========================================================
    // Phase 1: 盤面全体の超高速隣接マージ（同色限定、コスト1手）
    // =========================================================
    for &color in color_order {
        let mut merged = true;
        while merged {
            merged = false;
            let mut positions = [0usize; 400];
            let mut pos_count = 0;
            for p in 0..n * n {
                if board.last(p) == Some(color) {
                    positions[pos_count] = p;
                    pos_count += 1;
                }
            }
            if pos_count < 2 {
                break;
            }

            let target_plate = plates[color];
            for i in 0..pos_count {
                let p = positions[i];
                if board.last(p) != Some(color) { continue; }
                let h_p = board.len(p);

                for j in 0..pos_count {
                    if i == j { continue; }
                    let q = positions[j];
                    if board.last(q) != Some(color) { continue; }
                    let h_q = board.len(q);

                    if h_p + h_q <= target_tower_limit && dist_table[p][q] == 1 {
                        // 巣に近い方 q に向かって p が合体
                        let d_p = dist_table[p][target_plate];
                        let d_q = dist_table[q][target_plate];
                        let (from, to) = if d_p >= d_q { (p, q) } else { (q, p) };
                        if try_merge_adjacent_fast(n, dir_step, &mut board, plates, &mut ops, from, to) {
                            merged = true;
                            break;
                        }
                    }
                }
                if merged { break; }
            }
        }
    }

    // =========================================================
    // Phase 2: 各色の階層マージ（距離 2〜max_merge_dist の貪欲合体）
    // =========================================================
    for &color in color_order {
        let target_plate = plates[color];
        let mut loop_cnt = 0;

        while loop_cnt < 30 {
            loop_cnt += 1;
            let mut positions = [0usize; 400];
            let mut pos_count = 0;
            for p in 0..n * n {
                if board.last(p) == Some(color) {
                    positions[pos_count] = p;
                    pos_count += 1;
                }
            }
            if pos_count < 2 {
                break;
            }

            // 最良のペア (p, q) を探索
            // p を q に合流させる
            let mut best_pair = None;
            let mut best_score = isize::MIN;

            for i in 0..pos_count {
                let p = positions[i];
                let h_p = board.len(p);
                let d_p = dist_table[p][target_plate];
                if d_p == usize::MAX { continue; }

                for j in 0..pos_count {
                    if i == j { continue; }
                    let q = positions[j];
                    let h_q = board.len(q);
                    if h_p + h_q > target_tower_limit { continue; }

                    let d_pq = dist_table[p][q];
                    if d_pq == usize::MAX || d_pq > max_merge_dist { continue; }

                    let d_q = dist_table[q][target_plate];
                    if d_q == usize::MAX { continue; }

                    // スコア計算：
                    // p が巣に行くコスト d_p と比較して、p が q に行くコスト d_pq を引いたゲイン
                    // q が巣に近い（d_q <= d_p）なら進行方向ボーナス
                    let mut gain = (d_p as isize) - (d_pq as isize);
                    if d_q <= d_p {
                        gain += 4;
                    }
                    // 距離が近ければ近いほど確実で低コスト
                    gain -= (d_pq as isize) * 2;
                    // 高さがまとまるほど高評価
                    gain += (h_q as isize) * 2;

                    if gain > best_score {
                        best_score = gain;
                        best_pair = Some((p, q));
                    }
                }
            }

            let Some((from_p, to_q)) = best_pair else {
                break;
            };

            let success = move_tower_to_target(n, dir_step, max_ray, &mut board, plates, &mut ops, from_p, to_q, ctx);
            if !success {
                break;
            }
        }
    }

    // =========================================================
    // Phase 3: タワーの高速帰巣（巣への一撃射出ダンク＆タワー移動）
    // =========================================================
    for &color in color_order {
        let target_plate = plates[color];

        let mut round = 0;
        while round < 50 {
            round += 1;
            let mut positions = [0usize; 400];
            let mut pos_count = 0;
            for p in 0..n * n {
                if board.last(p) == Some(color) {
                    positions[pos_count] = p;
                    pos_count += 1;
                }
            }
            if pos_count == 0 {
                // 埋まっているスライムの開放チェック
                let mut buried = None;
                for p in 0..n * n {
                    if board.contains(p, color) {
                        buried = Some(p);
                        break;
                    }
                }
                if let Some(bp) = buried {
                    let h = board.len(bp);
                    let (top_len, _) = {
                        let l = board.len(bp);
                        let top_c = ((board.colors[bp] >> ((l - 1) * 4)) & 0xF) as usize;
                        let mut cnt = 0;
                        let mut shift = (l - 1) * 4;
                        for _ in 0..l {
                            if ((board.colors[bp] >> shift) & 0xF) as usize == top_c {
                                cnt += 1;
                                if shift < 4 { break; }
                                shift -= 4;
                            } else { break; }
                        }
                        (cnt, top_c)
                    };
                    let k_rem = h - top_len;
                    let mut ejected = false;
                    for d in 0..4 {
                        if max_ray[bp][d] >= 1 {
                            let np = (bp as isize + dir_step[d]) as usize;
                            if board.len(np) + top_len <= 8 {
                                ops.push(Op {
                                    r: bp / n,
                                    c: bp % n,
                                    k: k_rem,
                                    d: DIR_CHAR[d],
                                    l: 1,
                                });
                                board.transfer_and_reverse(bp, np, k_rem);
                                for &chk in &[bp, np] {
                                    if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                                        board.clean_nest(chk, pc);
                                    }
                                }
                                ejected = true;
                                break;
                            }
                        }
                    }
                    if ejected {
                        continue;
                    }
                }
                break;
            }

            // 高さの高いタワーから順に巣へ向かう
            positions[..pos_count].sort_by_key(|&p| {
                let h = board.len(p);
                let d = dist_table[p][target_plate];
                // 高さが高いほど優先、同じなら巣に近い方を優先
                (usize::MAX - h, d)
            });

            let best_p = positions[0];

            // best_p から巣へ向けて移動・ダンク！
            let success = move_tower_to_target(n, dir_step, max_ray, &mut board, plates, &mut ops, best_p, target_plate, ctx);
            if !success {
                // 通常の合体歩行で届かない場合、1歩でも巣に近づける
                let d_curr = dist_table[best_p][target_plate];
                let mut stepped = false;
                for d in 0..4 {
                    if max_ray[best_p][d] >= 1 {
                        let np = (best_p as isize + dir_step[d]) as usize;
                        if dist_table[np][target_plate] < d_curr && board.len(np) + board.len(best_p) <= 8 {
                            ops.push(Op {
                                r: best_p / n,
                                c: best_p % n,
                                k: 0,
                                d: DIR_CHAR[d],
                                l: 1,
                            });
                            board.transfer_and_reverse(best_p, np, 0);
                            for &chk in &[best_p, np] {
                                if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                                    board.clean_nest(chk, pc);
                                }
                            }
                            stepped = true;
                            break;
                        }
                    }
                }
                if !stepped {
                    break;
                }
            }
        }
    }

    // =========================================================
    // Phase 4: 取り残された全スライムの最終回収（フォールバック）
    // =========================================================
    let mut fallback_loops = 0;
    loop {
        fallback_loops += 1;
        if fallback_loops > 100 {
            break;
        }
        let mut remaining_p = None;
        for p in 0..n * n {
            if !board.is_empty(p) {
                remaining_p = Some(p);
                break;
            }
        }
        let Some(p) = remaining_p else {
            break;
        };

        let color = board.last(p).unwrap();
        let target_plate = plates[color];
        let success = move_tower_to_target(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, ctx);
        if !success {
            // 巣に届かない場合、任意の空きマスや隣接へ押し出す
            let mut moved = false;
            for d in 0..4 {
                if max_ray[p][d] >= 1 {
                    let np = (p as isize + dir_step[d]) as usize;
                    if board.len(np) + 1 <= 8 {
                        let k = board.len(p) - 1;
                        ops.push(Op {
                            r: p / n,
                            c: p % n,
                            k,
                            d: DIR_CHAR[d],
                            l: 1,
                        });
                        board.transfer_and_reverse(p, np, k);
                        for &chk in &[p, np] {
                            if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                                board.clean_nest(chk, pc);
                            }
                        }
                        moved = true;
                        break;
                    }
                }
            }
            if !moved {
                break;
            }
        }
    }

    let mut unreturned = 0;
    for p in 0..n * n {
        unreturned += board.len(p);
    }
    let score = ops.len() + unreturned * 100_000;
    (score, ops)
}

fn main() {
    let start_time = Instant::now();
    let total_time_limit = std::time::Duration::from_millis(1850);

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace();

    let n: usize = tokens.next().unwrap().parse().unwrap();
    let k: usize = tokens.next().unwrap().parse().unwrap();

    let mut floor = vec![false; n * n];
    let mut plates = vec![usize::MAX; k];
    let mut init_stacks = vec![Vec::<usize>::new(); n * n];

    for r in 0..n {
        let line = tokens.next().unwrap();
        for (c, ch) in line.chars().enumerate() {
            let u = r * n + c;
            match ch {
                '#' => floor[u] = false,
                '.' => floor[u] = true,
                'a'..='l' => {
                    floor[u] = true;
                    init_stacks[u].push((ch as u8 - b'a') as usize);
                }
                'A'..='L' => {
                    floor[u] = true;
                    let color = (ch as u8 - b'A') as usize;
                    plates[color] = u;
                }
                _ => {}
            }
        }
    }

    let dir_step = [-(n as isize), n as isize, -1, 1];
    let max_ray = compute_max_ray(n, &floor);
    let dist_table = compute_all_pairs_dist(n, &floor);
    let init_board = BitBoard::new(n, &init_stacks);

    let mut ctx = BfsContext::new();
    let mut rng = XorShift::new(123456789);

    // ---------------------------------------------------------
    // Phase 1: 初期オーダー候補の生成
    // ---------------------------------------------------------
    let mut order_candidates: Vec<Vec<usize>> = Vec::new();
    order_candidates.push((0..k).collect());

    let mut color_dists = Vec::new();
    for c in 0..k {
        let mut d_sum = 0;
        let mut cnt = 0;
        let target = plates[c];
        for p in 0..n * n {
            if init_board.contains(p, c) {
                let d = dist_table[p][target];
                if d != usize::MAX {
                    d_sum += d;
                    cnt += 1;
                }
            }
        }
        let avg_d = if cnt > 0 { d_sum as f64 / cnt as f64 } else { 0.0 };
        color_dists.push((c, avg_d));
    }

    // 巣から遠い順
    let mut dist_desc = color_dists.clone();
    dist_desc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    order_candidates.push(dist_desc.iter().map(|&(c, _)| c).collect());

    // 巣から近い順
    let mut dist_asc = color_dists.clone();
    dist_asc.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    order_candidates.push(dist_asc.iter().map(|&(c, _)| c).collect());

    // スライム数が多い順
    let mut color_counts = Vec::new();
    for c in 0..k {
        color_counts.push((c, init_board.count_color(c)));
    }
    let mut count_desc = color_counts.clone();
    count_desc.sort_by(|a, b| b.1.cmp(&a.1));
    order_candidates.push(count_desc.iter().map(|&(c, _)| c).collect());

    // ランダムシャッフル
    for _ in 0..4 {
        let mut shuf = (0..k).collect::<Vec<_>>();
        for i in (1..k).rev() {
            let j = rng.gen_range(0, i + 1);
            shuf.swap(i, j);
        }
        order_candidates.push(shuf);
    }

    let mut best_score = usize::MAX;
    let mut best_ops = Vec::new();
    let mut best_order = order_candidates[0].clone();
    let mut best_max_merge_dist = 5;
    let mut best_tower_limit = 8;

    for order in &order_candidates {
        for &dist in &[3, 5, 8] {
            for &t_lim in &[6, 8] {
                let (score, ops) = solve_hierarchical(
                    n,
                    k,
                    &dir_step,
                    &max_ray,
                    &floor,
                    &init_board,
                    &plates,
                    &dist_table,
                    order,
                    dist,
                    t_lim,
                    &mut ctx,
                );
                if score < best_score {
                    best_score = score;
                    best_ops = ops;
                    best_order = order.clone();
                    best_max_merge_dist = dist;
                    best_tower_limit = t_lim;
                }
            }
        }
    }

    // ---------------------------------------------------------
    // Phase 2: 焼きなまし / 多スタート局所探索
    // ---------------------------------------------------------
    let mut current_order = best_order;
    let mut current_dist = best_max_merge_dist;
    let mut current_t_lim = best_tower_limit;
    let mut current_score = best_score;

    let t_start: f64 = 10.0;
    let t_end: f64 = 0.5;
    let mut iters = 0;

    while start_time.elapsed() < total_time_limit {
        iters += 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let total = total_time_limit.as_secs_f64();
        let elapsed_ratio = (elapsed / total).min(1.0);
        let temperature = t_start * (1.0 - elapsed_ratio) + t_end * elapsed_ratio;

        let mut next_order = current_order.clone();
        let mut next_dist = current_dist;
        let mut next_t_lim = current_t_lim;

        let r = rng.gen_range(0, 100);
        if r < 30 {
            let i = rng.gen_range(0, k);
            let j = rng.gen_range(0, k);
            next_order.swap(i, j);
        } else if r < 60 {
            let from = rng.gen_range(0, k);
            let to = rng.gen_range(0, k);
            let val = next_order.remove(from);
            next_order.insert(to, val);
        } else if r < 80 {
            let mut i = rng.gen_range(0, k);
            let mut j = rng.gen_range(0, k);
            if i > j { std::mem::swap(&mut i, &mut j); }
            next_order[i..=j].reverse();
        } else if r < 90 {
            next_dist = rng.gen_range(2, 10);
        } else {
            next_t_lim = rng.gen_range(4, 9);
        }

        let (cand_score, cand_ops) = solve_hierarchical(
            n,
            k,
            &dir_step,
            &max_ray,
            &floor,
            &init_board,
            &plates,
            &dist_table,
            &next_order,
            next_dist,
            next_t_lim,
            &mut ctx,
        );

        let delta = cand_score as f64 - current_score as f64;
        let accept = if delta <= 0.0 {
            true
        } else {
            let prob = (-delta / temperature).exp();
            rng.gen_f64() < prob
        };

        if accept {
            current_order = next_order;
            current_dist = next_dist;
            current_t_lim = next_t_lim;
            current_score = cand_score;

            if cand_score < best_score {
                best_score = cand_score;
                best_ops = cand_ops;
            }
        }
    }

    eprintln!(
        "STAT: beam_iters=0 sa_iters={} winner=HierarchicalClustering final_score={}",
        iters, best_score
    );

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &best_ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
