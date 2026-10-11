// AHC 072 - Step 61: 動的最遠スライム優先・求心型タワー合体輸送（タワー完全一体化輸送）
// - タワーを1匹ずつ分解せず、タワー全員 (k=0) でまとまって移動・前進
// - 途中のスライムを雪だるま式に巻き込みながら巣へ向かう
// - 異色混載は最大3色まで厳格制限
// - 軸が合ったら土台を残して巣へダイレクトダンク ＆ 道中射出
// - 直前マスの逆走ピンポンを完全禁止
// - Step 55 の 100% AC 保証フォールバックを完備

use std::io::{self, Read};
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
                c_bits |= (init_stacks[p][i] as u32) << (i * 4);
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
            let val = ((self.colors[p] >> shift) & 0xF) as usize;
            Some(val)
        }
    }

    #[inline(always)]
    fn contains(&self, p: usize, color: usize) -> bool {
        let l = self.lens[p] as usize;
        let mut c = self.colors[p];
        for _ in 0..l {
            if (c & 0xF) as usize == color {
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
        let mask = if block_len >= 8 { 0xFFFF_FFFF } else { (1u32 << (block_len * 4)) - 1 };
        let block = (self.colors[p] >> shift_k) & mask;

        self.lens[p] = k as u8;
        self.colors[p] &= if k == 0 { 0 } else { (1u32 << shift_k) - 1 };

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
    fn count_top_consecutive(&self, p: usize) -> (usize, usize) {
        let l = self.lens[p] as usize;
        if l == 0 {
            return (0, 0);
        }
        let top_color = ((self.colors[p] >> ((l - 1) * 4)) & 0xF) as usize;
        let mut count = 0;
        let mut shift = (l - 1) * 4;
        for _ in 0..l {
            if ((self.colors[p] >> shift) & 0xF) as usize == top_color {
                count += 1;
                if shift < 4 { break; }
                shift -= 4;
            } else {
                break;
            }
        }
        (count, top_color)
    }

    #[inline(always)]
    fn unique_color_mask(&self, p: usize) -> u32 {
        let l = self.lens[p] as usize;
        let mut c = self.colors[p];
        let mut mask = 0u32;
        for _ in 0..l {
            mask |= 1 << (c & 0xF);
            c >>= 4;
        }
        mask
    }

    #[inline(always)]
    fn total_slimes(&self) -> usize {
        let mut sum = 0;
        for &l in &self.lens {
            sum += l as usize;
        }
        sum
    }
}

// -------------------------------------------------------------
// BFS 関連
// -------------------------------------------------------------
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

// タワー合体歩行 BFS: タワー全員 (start_h) で目標マスまで歩行探索
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

    while head < tail {
        let curr_idx = ctx.queue_tower[head];
        head += 1;

        let u = curr_idx / 9;
        let h = curr_idx % 9;

        if u == target {
            best_target_idx = Some(curr_idx);
            break;
        }

        let mut dirs = [0usize, 1, 2, 3];
        dirs.sort_by_key(|&d| {
            if max_ray[u][d] < 1 { return 3; }
            let v = (u as isize + dir_step[d]) as usize;
            let next_len = board.len(v);
            if next_len > 0 && h + next_len <= 8 {
                if board.last(v) == target_color {
                    0 // 同色スライム最優先
                } else {
                    1 // 異色スライム合体可能
                }
            } else if next_len == 0 {
                2 // 空きマス
            } else {
                3 // 高さ超過
            }
        });

        for &d in &dirs {
            if max_ray[u][d] < 1 {
                continue;
            }
            let v = (u as isize + dir_step[d]) as usize;

            let next_h = if v == target {
                if board.len(v) + h > 8 {
                    continue;
                }
                h
            } else {
                let nh = h + board.len(v);
                if nh > 8 {
                    continue;
                }
                // 混載色数チェック（3色以内）
                if (board.unique_color_mask(u) | board.unique_color_mask(v)).count_ones() > 3 {
                    continue;
                }
                nh
            };

            let next_idx = v * 9 + next_h;
            if ctx.visited_tower[next_idx] != token {
                ctx.visited_tower[next_idx] = token;
                ctx.dist_tower[next_idx] = ctx.dist_tower[curr_idx] + 1;
                ctx.prev_tower[next_idx] = (curr_idx, d);
                ctx.queue_tower[tail] = next_idx;
                tail += 1;
            }
        }
    }

    let Some(target_idx) = best_target_idx else {
        return None;
    };

    let mut step_count = 0;
    let mut curr_idx = target_idx;
    while curr_idx != start_idx && step_count < 400 {
        let (prev_idx, d) = ctx.prev_tower[curr_idx];
        ctx.path_tower[step_count] = d;
        step_count += 1;
        curr_idx = prev_idx;
    }
    ctx.path_tower[..step_count].reverse();
    Some(step_count)
}

// 単体・複数歩行・ジャンプによる転送パス探索（フォールバック用）
fn find_path_single_trampoline_fast(
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &BitBoard,
    start: usize,
    target: usize,
    ctx: &mut BfsContext,
) -> Option<usize> {
    ctx.token += 1;
    let token = ctx.token;

    ctx.visited[start] = token;
    ctx.dist[start] = 0;

    let mut head = 0;
    let mut tail = 0;
    ctx.queue[tail] = start;
    tail += 1;

    let mut reached = false;

    while head < tail {
        let u = ctx.queue[head];
        head += 1;

        if u == target {
            reached = true;
            break;
        }

        let h = if u == start {
            board.len(start)
        } else {
            board.len(u) + 1
        };

        if h == 0 {
            continue;
        }

        let k = h - 1;
        let max_l = k + 1;

        for d in 0..4 {
            let ray_limit = max_ray[u][d];
            let limit = max_l.min(ray_limit);
            let step = dir_step[d];
            let mut v = u as isize;

            for l in 1..=limit {
                v += step;
                let next_p = v as usize;

                if board.len(next_p) + 1 > 8 {
                    continue;
                }

                if ctx.visited[next_p] != token {
                    ctx.visited[next_p] = token;
                    ctx.dist[next_p] = ctx.dist[u] + 1;
                    ctx.prev[next_p] = u;
                    ctx.prev_action[next_p] = (k, d, l);
                    ctx.queue[tail] = next_p;
                    tail += 1;
                }
            }
        }
    }

    if !reached {
        return None;
    }

    let mut action_count = 0;
    let mut curr = target;
    while curr != start && action_count < 400 {
        let p = ctx.prev[curr];
        let (k, d, l) = ctx.prev_action[curr];
        ctx.actions[action_count] = (p, k, d, l);
        action_count += 1;
        curr = p;
    }
    ctx.actions[..action_count].reverse();
    Some(action_count)
}

fn move_smart_transfer(
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
    if start == target {
        return true;
    }
    let Some(action_count) = find_path_single_trampoline_fast(dir_step, max_ray, board, start, target, ctx) else {
        return false;
    };
    if action_count == 0 {
        return true;
    }

    for idx in 0..action_count {
        let (p, k, d, l) = ctx.actions[idx];
        let h = board.len(p);
        if h == 0 || k >= h {
            return false;
        }

        let next_p = (p as isize + dir_step[d] * l as isize) as usize;
        if board.len(next_p) + (h - k) > 8 {
            return false;
        }

        ops.push(Op {
            r: p / n,
            c: p % n,
            k,
            d: DIR_CHAR[d],
            l,
        });

        board.transfer_and_reverse(p, next_p, k);

        for &chk in &[p, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
                board.clean_nest(chk, plate_color);
            }
        }

        if board.is_empty(next_p) {
            break;
        }
    }
    true
}

// 巣ブロッカーの高速排出
fn eject_nest_blockers_fast(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    plate_pos: usize,
    plate_color: usize,
) -> bool {
    let mut ejected_any = false;
    while board.len(plate_pos) > 0 {
        let top_color = board.last(plate_pos).unwrap();
        if top_color == plate_color {
            board.clean_nest(plate_pos, plate_color);
            continue;
        }

        let (cnt, _) = board.count_top_consecutive(plate_pos);
        let h = board.len(plate_pos);
        let k = h - cnt;

        let mut best_d = None;
        let mut max_ray_len = 0;
        for d in 0..4 {
            let ray = max_ray[plate_pos][d];
            if ray >= 1 {
                let np = (plate_pos as isize + dir_step[d]) as usize;
                if !plates.contains(&np) && board.len(np) + cnt <= 8 {
                    if ray > max_ray_len {
                        max_ray_len = ray;
                        best_d = Some(d);
                    }
                }
            }
        }

        let Some(d) = best_d else {
            break;
        };

        let next_p = (plate_pos as isize + dir_step[d]) as usize;
        ops.push(Op {
            r: plate_pos / n,
            c: plate_pos % n,
            k,
            d: DIR_CHAR[d],
            l: 1,
        });
        board.transfer_and_reverse(plate_pos, next_p, k);
        ejected_any = true;
    }
    ejected_any
}

// 道中射出（Cross-color ejection）
fn try_cross_color_ejection_fast(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    p: usize,
) -> bool {
    let h = board.len(p);
    if h < 2 {
        return false;
    }

    let top_color = board.last(p).unwrap();
    let nest = plates[top_color];
    if p == nest {
        return false;
    }

    let r_p = p / n;
    let c_p = p % n;
    let r_t = nest / n;
    let c_t = nest % n;

    let on_axis = if r_p == r_t {
        if c_t > c_p { Some((3, c_t - c_p)) } else { Some((2, c_p - c_t)) }
    } else if c_p == c_t {
        if r_t > r_p { Some((1, r_t - r_p)) } else if r_p > r_t { Some((0, r_p - r_t)) } else { None }
    } else {
        None
    };

    if let Some((d, dist_l)) = on_axis {
        if max_ray[p][d] >= dist_l {
            if dist_l > 1 && h >= dist_l {
                let k_rem = dist_l - 1;
                let flying = h - k_rem;
                if flying > 0 && board.len(nest) + flying <= 8 {
                    ops.push(Op {
                        r: r_p,
                        c: c_p,
                        k: k_rem,
                        d: DIR_CHAR[d],
                        l: dist_l,
                    });
                    board.transfer_and_reverse(p, nest, k_rem);
                    board.clean_nest(nest, top_color);
                    return true;
                }
            }
        }
    }

    false
}

// -------------------------------------------------------------
// 【核心関数】タワー全員で巣へ向かって進む（決して分解しない！）
// -------------------------------------------------------------
fn move_tower_forward(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    start: usize,
    target: usize,
    dist_table: &[Vec<usize>],
    ctx: &mut BfsContext,
) -> bool {
    let start_h = board.len(start);
    if start_h == 0 || start == target {
        return false;
    }

    if let Some(step_count) = find_path_merge_fast(
        dir_step, max_ray, board, plates, start, target, start_h, ctx,
    ) {
        let mut curr = start;
        for idx in 0..step_count {
            let d = ctx.path_tower[idx];
            let next_p = (curr as isize + dir_step[d]) as usize;

            let h_curr = board.len(curr);
            if h_curr == 0 {
                break;
            }
            if board.len(next_p) + h_curr > 8 {
                break;
            }

            let r = curr / n;
            let c = curr % n;
            ops.push(Op { r, c, k: 0, d: DIR_CHAR[d], l: 1 });
            board.transfer_and_reverse(curr, next_p, 0);

            for (c_idx, &nest_pos) in plates.iter().enumerate() {
                if nest_pos == next_p {
                    board.clean_nest(nest_pos, c_idx);
                }
            }

            curr = next_p;

            // 移動途中、頭に目的の色のスライムがあり巣と軸が合えば道中ダンク！
            if board.len(curr) >= 2 {
                try_cross_color_ejection_fast(n, dir_step, max_ray, board, plates, ops, curr);
            }

            if curr == target || board.is_empty(curr) {
                break;
            }
        }
        return true;
    }

    // もし BFS で経路が見つからない場合：巣へ近づく 1 歩（空きマス）を試みる
    let curr_d = dist_table[start][target];
    for d in 0..4 {
        if max_ray[start][d] < 1 {
            continue;
        }
        let np = (start as isize + dir_step[d]) as usize;
        let h_curr = board.len(start);
        if board.len(np) + h_curr <= 8 && dist_table[np][target] < curr_d {
            let r = start / n;
            let c = start % n;
            ops.push(Op { r, c, k: 0, d: DIR_CHAR[d], l: 1 });
            board.transfer_and_reverse(start, np, 0);
            for (c_idx, &nest_pos) in plates.iter().enumerate() {
                if nest_pos == np {
                    board.clean_nest(nest_pos, c_idx);
                }
            }
            return true;
        }
    }

    false
}

// -------------------------------------------------------------
// 【核心評価関数】隣接マスへの全員移動・合体の評価 (k=0, l=1)
// -------------------------------------------------------------
fn evaluate_adjacent_move_tolerant(
    board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    p: usize,
    next_p: usize,
    max_tol_per_slime: usize,
) -> Option<isize> {
    let h_p = board.len(p);
    let h_next = board.len(next_p);

    if h_p + h_next > 8 {
        return None;
    }

    // 混載色数のチェック（合体後 3 色以内）
    let mask_p = board.unique_color_mask(p);
    let mask_next = board.unique_color_mask(next_p);
    let combined_mask = mask_p | mask_next;
    if combined_mask.count_ones() > 3 {
        return None;
    }

    // p にいる全員の巣までの距離変化をチェック
    let mut total_dist_diff: isize = 0;
    let mut bits = board.colors[p];
    for _ in 0..h_p {
        let color = (bits & 0xF) as usize;
        bits >>= 4;
        let nest = plates[color];
        let d_curr = dist_table[p][nest];
        let d_next = dist_table[next_p][nest];
        if d_curr == usize::MAX || d_next == usize::MAX {
            return None;
        }

        // 許容幅を超えて巣から遠ざかる移動は禁止
        if d_next > d_curr + max_tol_per_slime {
            return None;
        }

        total_dist_diff += (d_curr as isize) - (d_next as isize);
    }

    if h_next > 0 {
        // 合体の場合：相手のスライムを巻き込めるので高ボーナス
        let score = total_dist_diff * 15 + (h_next as isize) * 30 + 50;
        Some(score)
    } else {
        // 空きマスへの前進：巣に近づく移動（total_dist_diff > 0）のみ許可
        if total_dist_diff > 0 {
            let score = total_dist_diff * 10;
            Some(score)
        } else {
            None
        }
    }
}

// -------------------------------------------------------------
// 【新ソルバー】動的最遠優先 ＆ 求心型タワー合体輸送
// -------------------------------------------------------------
fn solve_centripetal_cohesive(
    n: usize,
    k: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    init_board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    max_tol_per_slime: usize,
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(500);

    // 直前の移動の記録（ピンポン防止: (from, to)）
    let mut last_move: Option<(usize, usize)> = None;

    // Stage 1: 直撃バイパス（最初から巣に跳べるスライムを一撃ダンク）
    for p in 0..n * n {
        if board.is_empty(p) {
            continue;
        }
        if let Some(c) = board.last(p) {
            let target = plates[c];
            if p == target {
                continue;
            }
            let r_p = p / n;
            let c_p = p % n;
            let r_t = target / n;
            let c_t = target % n;

            let on_axis = if r_p == r_t {
                if c_t > c_p { Some((3, c_t - c_p)) } else { Some((2, c_p - c_t)) }
            } else if c_p == c_t {
                if r_t > r_p { Some((1, r_t - r_p)) } else if r_p > r_t { Some((0, r_p - r_t)) } else { None }
            } else {
                None
            };

            if let Some((d, dist_l)) = on_axis {
                if max_ray[p][d] >= dist_l {
                    let h = board.len(p);
                    if dist_l == 1 && board.len(target) + h <= 8 {
                        ops.push(Op { r: r_p, c: c_p, k: 0, d: DIR_CHAR[d], l: 1 });
                        board.transfer_and_reverse(p, target, 0);
                        board.clean_nest(target, c);
                    } else if dist_l > 1 && h >= dist_l {
                        let k_rem = dist_l - 1;
                        if board.len(target) + (h - k_rem) <= 8 {
                            ops.push(Op { r: r_p, c: c_p, k: k_rem, d: DIR_CHAR[d], l: dist_l });
                            board.transfer_and_reverse(p, target, k_rem);
                            board.clean_nest(target, c);
                        }
                    }
                }
            }
        }
    }

    // Stage 2: 動的最遠スライム優先・求心型ループ
    let mut step_iter = 0;
    while board.total_slimes() > 0 && step_iter < 1200 {
        step_iter += 1;

        // 全盤面で巣の上にあるスライムを消去
        for c in 0..k {
            board.clean_nest(plates[c], c);
        }
        if board.total_slimes() == 0 {
            break;
        }

        // 道中射出をチェック
        let mut ejected_any = false;
        for p in 0..n * n {
            if board.len(p) >= 2 {
                if try_cross_color_ejection_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p) {
                    ejected_any = true;
                }
            }
        }
        if ejected_any {
            continue;
        }

        // 盤面で「最も巣から遠いスライム」を動的に探索
        let mut best_p = None;
        let mut max_dist = 0;
        let mut best_h = 0;

        for p in 0..n * n {
            if board.is_empty(p) {
                continue;
            }
            if let Some(c) = board.last(p) {
                let nest = plates[c];
                let d = dist_table[p][nest];
                if d != usize::MAX {
                    let h = board.len(p);
                    if d > max_dist || (d == max_dist && h > best_h) {
                        max_dist = d;
                        best_h = h;
                        best_p = Some(p);
                    }
                }
            }
        }

        let Some(p) = best_p else {
            break;
        };
        let top_c = board.last(p).unwrap();
        let nest = plates[top_c];
        let h_p = board.len(p);

        // 巣の直上にいる場合、巣ブロッカー排出
        if p == nest {
            eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, nest, top_c);
            continue;
        }

        // 候補A: 巣への直撃ダンクが可能か？
        let r_p = p / n;
        let c_p = p % n;
        let r_t = nest / n;
        let c_t = nest % n;

        let on_axis = if r_p == r_t {
            if c_t > c_p { Some((3, c_t - c_p)) } else { Some((2, c_p - c_t)) }
        } else if c_p == c_t {
            if r_t > r_p { Some((1, r_t - r_p)) } else if r_p > r_t { Some((0, r_p - r_t)) } else { None }
        } else {
            None
        };

        let mut launched = false;
        if let Some((d, dist_l)) = on_axis {
            if max_ray[p][d] >= dist_l {
                if dist_l == 1 && board.len(nest) + h_p <= 8 {
                    ops.push(Op { r: r_p, c: c_p, k: 0, d: DIR_CHAR[d], l: 1 });
                    board.transfer_and_reverse(p, nest, 0);
                    board.clean_nest(nest, top_c);
                    launched = true;
                    last_move = Some((p, nest));
                } else if dist_l > 1 && h_p >= dist_l {
                    let k_rem = dist_l - 1;
                    if board.len(nest) + (h_p - k_rem) <= 8 {
                        ops.push(Op { r: r_p, c: c_p, k: k_rem, d: DIR_CHAR[d], l: dist_l });
                        board.transfer_and_reverse(p, nest, k_rem);
                        board.clean_nest(nest, top_c);
                        launched = true;
                        last_move = Some((p, nest));
                    }
                }
            }
        }
        if launched {
            continue;
        }

        // 候補B: 隣接マスへの全員合体 (k=0, l=1)
        let mut best_action = None;
        let mut best_score = isize::MIN;

        for d in 0..4 {
            if max_ray[p][d] < 1 {
                continue;
            }
            let np = (p as isize + dir_step[d]) as usize;

            // ピンポン防止
            if let Some((prev_from, prev_to)) = last_move {
                if prev_to == p && prev_from == np {
                    continue;
                }
            }

            if let Some(score) = evaluate_adjacent_move_tolerant(
                &board,
                plates,
                dist_table,
                p,
                np,
                max_tol_per_slime,
            ) {
                if score > best_score {
                    best_score = score;
                    best_action = Some((d, np));
                }
            }
        }

        if let Some((d, np)) = best_action {
            ops.push(Op { r: r_p, c: c_p, k: 0, d: DIR_CHAR[d], l: 1 });
            board.transfer_and_reverse(p, np, 0);
            for (c, &nest_pos) in plates.iter().enumerate() {
                if nest_pos == np {
                    board.clean_nest(nest_pos, c);
                }
            }
            last_move = Some((p, np));
            continue;
        }

        // 候補C: タワー全員で巣へ向かって進む（決して分解しない！）
        let moved = move_tower_forward(n, dir_step, max_ray, &mut board, plates, &mut ops, p, nest, dist_table, ctx);
        if moved {
            last_move = None;
            continue;
        }

        // それでも動けなければ、巣のブロッカー排出などを試みる
        eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, nest, top_c);
    }

    // Stage 3: 安全フォールバック（Step 55 の 100% AC 保証構造）
    let mut outer = 0;
    while board.total_slimes() > 0 && outer < 25 {
        outer += 1;
        let mut progress = false;

        for color in 0..k {
            let target = plates[color];
            for p in 0..n * n {
                if board.contains(p, color) {
                    if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target, ctx) {
                        progress = true;
                    }
                    eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, target, color);
                }
            }
        }

        // 道中射出
        for p in 0..n * n {
            if board.len(p) >= 2 {
                if try_cross_color_ejection_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p) {
                    progress = true;
                }
            }
        }

        // 埋没スライムの露出排出
        if !progress {
            for color in 0..k {
                let mut buried_p = None;
                for p in 0..n * n {
                    if board.contains(p, color) && board.last(p) != Some(color) {
                        buried_p = Some(p);
                        break;
                    }
                }
                if let Some(bp) = buried_p {
                    let (cnt, _) = board.count_top_consecutive(bp);
                    let h = board.len(bp);
                    let k_rem = h - cnt;
                    for d in 0..4 {
                        if max_ray[bp][d] < 1 {
                            continue;
                        }
                        let np = (bp as isize + dir_step[d]) as usize;
                        if board.len(np) + cnt <= 8 {
                            ops.push(Op {
                                r: bp / n,
                                c: bp % n,
                                k: k_rem,
                                d: DIR_CHAR[d],
                                l: 1,
                            });
                            board.transfer_and_reverse(bp, np, k_rem);
                            progress = true;
                            break;
                        }
                    }
                }
                if progress {
                    break;
                }
            }
        }

        // 単体強制移動
        if !progress {
            for p in 0..n * n {
                if let Some(c) = board.last(p) {
                    let target = plates[c];
                    if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target, ctx) {
                        progress = true;
                    }
                    eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, target, c);
                    break;
                }
            }
        }

        if !progress {
            break;
        }
    }

    for c in 0..k {
        board.clean_nest(plates[c], c);
    }

    (ops.len(), ops)
}

fn main() {
    let start_time = Instant::now();
    let total_time_limit = std::time::Duration::from_millis(1850);

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut lines = input.lines();

    let first_line = lines.next().unwrap();
    let mut parts = first_line.split_whitespace();
    let n: usize = parts.next().unwrap().parse().unwrap();
    let k: usize = parts.next().unwrap().parse().unwrap();

    let mut floor = vec![true; n * n];
    let mut init_stacks: Vec<Vec<usize>> = vec![Vec::new(); n * n];
    let mut plates = vec![0; k];

    for i in 0..n {
        let line = lines.next().unwrap();
        for (j, ch) in line.chars().enumerate() {
            let p = i * n + j;
            match ch {
                '#' => floor[p] = false,
                '.' => {}
                'a'..='z' => {
                    let color = (ch as u8 - b'a') as usize;
                    init_stacks[p].push(color);
                }
                'A'..='Z' => {
                    let color = (ch as u8 - b'A') as usize;
                    plates[color] = p;
                }
                _ => {}
            }
        }
    }

    let dir_step: [isize; 4] = [-(n as isize), n as isize, -1, 1];
    let max_ray = compute_max_ray(n, &floor);
    let dist_table = compute_all_pairs_dist(n, &floor);
    let init_board = BitBoard::new(n, &init_stacks);
    let mut ctx = Box::new(BfsContext::new());

    // 許容度 (max_tol) を 1, 2, 0 で評価し、最善を採用
    let tol_candidates = [1, 2, 0];
    let mut best_score = usize::MAX;
    let mut best_ops = Vec::new();
    let mut eval_count = 0;

    for &tol in &tol_candidates {
        if start_time.elapsed() >= total_time_limit {
            break;
        }
        eval_count += 1;
        let (score, ops) = solve_centripetal_cohesive(
            n,
            k,
            &dir_step,
            &max_ray,
            &init_board,
            &plates,
            &dist_table,
            tol,
            &mut ctx,
        );
        if score < best_score {
            best_score = score;
            best_ops = ops;
        }
    }

    eprintln!("STAT: final_score={} cfg_evals={}", best_score, eval_count);

    for op in best_ops {
        println!("{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l);
    }
}
