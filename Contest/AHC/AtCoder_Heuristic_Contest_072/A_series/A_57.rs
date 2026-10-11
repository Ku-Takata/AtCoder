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
    fn pop(&mut self, p: usize) -> Option<usize> {
        let l = self.lens[p] as usize;
        if l == 0 {
            None
        } else {
            let shift = (l - 1) * 4;
            let val = ((self.colors[p] >> shift) & 0xF) as usize;
            self.lens[p] -= 1;
            let new_l = self.lens[p] as usize;
            self.colors[p] &= if new_l == 0 { 0 } else { (1u32 << (new_l * 4)) - 1 };
            Some(val)
        }
    }

    #[inline(always)]
    fn unique_color_mask(&self, p: usize) -> u16 {
        let l = self.lens[p] as usize;
        if l == 0 {
            return 0;
        }
        let mut mask = 0u16;
        let mut c = self.colors[p];
        for _ in 0..l {
            mask |= 1 << (c & 0xF);
            c >>= 4;
        }
        mask
    }

    #[inline(always)]
    fn count_unique_colors(&self, p: usize) -> usize {
        self.unique_color_mask(p).count_ones() as usize
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

    // 複数同色スライムを一括で巣に吸収・消去
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

    #[inline(always)]
    fn total_slimes(&self) -> usize {
        let mut t = 0;
        for p in 0..400 {
            t += self.lens[p] as usize;
        }
        t
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
            while r + l + 1 < n && floor[(r + 1 + l) * n + c] {
                l += 1;
            }
            max_ray[u][1] = l;

            let mut l = 0;
            while c >= l + 1 && floor[r * n + (c - 1 - l)] {
                l += 1;
            }
            max_ray[u][2] = l;

            let mut l = 0;
            while c + l + 1 < n && floor[r * n + (c + 1 + l)] {
                l += 1;
            }
            max_ray[u][3] = l;
        }
    }
    max_ray
}

fn compute_all_pairs_dist(n: usize, floor: &[bool]) -> Vec<Vec<usize>> {
    let sz = n * n;
    let mut d = vec![vec![usize::MAX; sz]; sz];
    for i in 0..sz {
        if !floor[i] {
            continue;
        }
        d[i][i] = 0;
        let r = i / n;
        let c = i % n;
        for (dr, dc) in DIR {
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            if nr >= 0 && nr < n as isize && nc >= 0 && nc < n as isize {
                let j = (nr as usize) * n + (nc as usize);
                if floor[j] {
                    d[i][j] = 1;
                }
            }
        }
    }
    for k in 0..sz {
        if !floor[k] {
            continue;
        }
        for i in 0..sz {
            if !floor[i] || d[i][k] == usize::MAX {
                continue;
            }
            for j in 0..sz {
                if !floor[j] || d[k][j] == usize::MAX {
                    continue;
                }
                let val = d[i][k] + d[k][j];
                if val < d[i][j] {
                    d[i][j] = val;
                }
            }
        }
    }
    d
}

#[inline(always)]
fn do_op(
    n: usize,
    dir_step: &[isize; 4],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    p: usize,
    k: usize,
    d: usize,
    l: usize,
) -> usize {
    let r = p / n;
    let c = p % n;
    ops.push(Op {
        r,
        c,
        k,
        d: DIR_CHAR[d],
        l,
    });
    let next_p = (p as isize + dir_step[d] * l as isize) as usize;
    board.transfer_and_reverse(p, next_p, k);
    for &chk in &[p, next_p] {
        if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
            board.clean_nest(chk, plate_color);
        }
    }
    next_p
}

// -------------------------------------------------------------
// 高速単体トランポリン探索 BFS
// -------------------------------------------------------------
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

        // 1. 基本の1歩歩行 (k=0, l=1) は、壁でなければいつでも可能！
        for d in 0..4 {
            if max_ray[u][d] < 1 {
                continue;
            }
            let next_p = (u as isize + dir_step[d]) as usize;
            if board.len(next_p) + 1 > 8 {
                continue;
            }
            if ctx.visited[next_p] != token {
                ctx.visited[next_p] = token;
                ctx.dist[next_p] = ctx.dist[u] + 1;
                ctx.prev[next_p] = u;
                ctx.prev_action[next_p] = (0, d, 1);
                ctx.queue[tail] = next_p;
                tail += 1;
            }
        }

        // 2. もし u に元々タワーがあるなら、大ジャンプ (k > 0, l > 1) も可能！
        if board.len(u) >= 2 {
            let h = board.len(u);
            let k = h - 1;
            let max_l = (k + 1).min(8);

            for d in 0..4 {
                let ray_limit = max_ray[u][d];
                let limit = max_l.min(ray_limit);
                let step = dir_step[d];
                let mut v = u as isize;

                for l in 2..=limit {
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

fn move_single_trampoline_fast(
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

        do_op(n, dir_step, board, plates, ops, p, k, d, l);

        if board.is_empty(next_p) {
            break;
        }
    }
    true
}

// -------------------------------------------------------------
// タワー合体歩行 BFS（find_path_merge_fast）
// 途中マスにスライムがいても、合体して <= 8 かつ 混載 <= 3色 なら通過・吸収！
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

// -------------------------------------------------------------
// 統合移動エンジン（move_smart_transfer）
// -------------------------------------------------------------
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
    if start == target || board.is_empty(start) {
        return true;
    }

    let h = board.len(start);
    if h == 1 {
        return move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, start, target, ctx);
    }

    let r_s = start / n;
    let c_s = start % n;
    let r_t = target / n;
    let c_t = target % n;

    // 直線大ジャンプ（途中のスライムタワーを飛び越え）
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
            if dist_l == 1 && board.len(target) + h <= 8 {
                do_op(n, dir_step, board, plates, ops, start, 0, d, 1);
                return true;
            }
            if dist_l > 1 && h >= dist_l && plates.contains(&target) {
                let k = dist_l - 1;
                let target_color = plates.iter().position(|&pos| pos == target).unwrap();
                let mut base_has_target = false;
                let mut c = board.colors[start];
                for _ in 0..k {
                    if ((c & 0xF) as usize) == target_color {
                        base_has_target = true;
                        break;
                    }
                    c >>= 4;
                }
                if !base_has_target {
                    let flying = h - k;
                    if board.len(target) + flying <= 8 {
                        do_op(n, dir_step, board, plates, ops, start, k, d, dist_l);
                        if board.is_empty(start) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // タワー合体歩行
    if let Some(step_count) = find_path_merge_fast(dir_step, max_ray, board, plates, start, target, h, ctx) {
        let mut curr = start;
        for idx in 0..step_count {
            let d = ctx.path_tower[idx];
            let next_p = (curr as isize + dir_step[d]) as usize;

            if board.len(next_p) + board.len(curr) > 8 {
                break;
            }
            do_op(n, dir_step, board, plates, ops, curr, 0, d, 1);

            curr = next_p;
            if curr == target || board.is_empty(curr) {
                break;
            }
        }
        return true;
    }

    // 単体・分割トランポリン輸送（フォールバック）
    let mut fallback_count = 0;
    while !board.is_empty(start) && fallback_count < 10 {
        let s = move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, start, target, ctx);
        if !s {
            break;
        }
        fallback_count += 1;
    }

    start == target || board.is_empty(start)
}

// -------------------------------------------------------------
// 巣ブロッカー排出（eject_nest_blockers_fast）
// -------------------------------------------------------------
fn eject_nest_blockers_fast(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    nest: usize,
    nest_color: usize,
) {
    let mut eject_count = 0;
    while !board.is_empty(nest) && eject_count < 10 {
        if board.last(nest) == Some(nest_color) {
            board.clean_nest(nest, nest_color);
            if board.is_empty(nest) {
                break;
            }
        }

        let (count, _) = board.count_top_consecutive(nest);
        let h = board.len(nest);
        let k = h - count;
        let max_l = (k + 1).min(8);

        let mut best_action = None;
        for d in 0..4 {
            let limit = max_l.min(max_ray[nest][d]);
            let step = dir_step[d];
            let mut v = nest as isize;
            for l in 1..=limit {
                v += step;
                let np = v as usize;
                if board.len(np) + count <= 8 {
                    best_action = Some((d, l));
                    break;
                }
            }
            if best_action.is_some() {
                break;
            }
        }

        let Some((d, l)) = best_action else {
            break;
        };

        do_op(n, dir_step, board, plates, ops, nest, k, d, l);
        board.clean_nest(nest, nest_color);
        eject_count += 1;
    }
}

// -------------------------------------------------------------
// 道中大ジャンプ射出（try_cross_color_ejection_fast）
// -------------------------------------------------------------
fn try_cross_color_ejection_fast(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    p: usize,
    dist_table: &[Vec<usize>],
) -> bool {
    let h = board.len(p);
    if h < 2 {
        return false;
    }

    let (top_count, top_color) = board.count_top_consecutive(p);
    let k = h - top_count;
    if k == 0 {
        return false;
    }

    let max_l = (k + 1).min(8);
    let target_plate = plates[top_color];
    let curr_dist_to_nest = dist_table[p][target_plate];
    let r_tgt = target_plate / n;
    let c_tgt = target_plate % n;

    let mut best_action = None;
    let mut best_gain = 0isize;

    for d in 0..4 {
        let limit = max_l.min(max_ray[p][d]);
        let step = dir_step[d];
        let mut v = p as isize;

        for l in 1..=limit {
            v += step;
            let next_p = v as usize;

            if board.len(next_p) + top_count > 8 {
                continue;
            }

            // 混載色数チェック（3色以内）
            let mut test_board = *board;
            test_board.transfer_and_reverse(p, next_p, k);
            if test_board.count_unique_colors(next_p) > 3 {
                continue;
            }

            let next_dist_to_nest = dist_table[next_p][target_plate];
            let landing_len = board.len(next_p);

            // ケース A: 巣に直撃ダンク！
            if next_p == target_plate {
                let gain = 10000 + (l as isize) * 10;
                if gain > best_gain {
                    best_gain = gain;
                    best_action = Some((d, l));
                }
                continue;
            }

            // ケース B: 同色のスライムがいるマスに着地（合体！）
            if landing_len > 0 && board.last(next_p) == Some(top_color) {
                let gain = 1000 + (l as isize) * 10;
                if gain > best_gain {
                    best_gain = gain;
                    best_action = Some((d, l));
                }
                continue;
            }

            // ケース C: 巣へ近づく大ジャンプ（l >= 2、かつ厳格に距離短縮）
            if l >= 2 && next_dist_to_nest < curr_dist_to_nest {
                let dist_saved = (curr_dist_to_nest - next_dist_to_nest) as isize;
                let r_next = next_p / n;
                let c_next = next_p % n;

                let mut true_on_axis = false;
                let mut on_axis_dist = 0;
                if r_next == r_tgt {
                    if c_tgt > c_next {
                        let dist = c_tgt - c_next;
                        if max_ray[next_p][3] >= dist {
                            true_on_axis = true;
                            on_axis_dist = dist;
                        }
                    } else if c_next > c_tgt {
                        let dist = c_next - c_tgt;
                        if max_ray[next_p][2] >= dist {
                            true_on_axis = true;
                            on_axis_dist = dist;
                        }
                    }
                } else if c_next == c_tgt {
                    if r_tgt > r_next {
                        let dist = r_tgt - r_next;
                        if max_ray[next_p][1] >= dist {
                            true_on_axis = true;
                            on_axis_dist = dist;
                        }
                    } else if r_next > r_tgt {
                        let dist = r_next - r_tgt;
                        if max_ray[next_p][0] >= dist {
                            true_on_axis = true;
                            on_axis_dist = dist;
                        }
                    }
                }

                let on_axis_bonus = if true_on_axis {
                    let mut b = 45;
                    if on_axis_dist <= 4 {
                        b += 25;
                    }
                    b
                } else if r_next == r_tgt || c_next == c_tgt {
                    15
                } else {
                    0
                };

                let gain = dist_saved * 15 + (l as isize) * 5 + on_axis_bonus;
                if gain > 15 && gain > best_gain {
                    best_gain = gain;
                    best_action = Some((d, l));
                }
            }
        }
    }

    if let Some((d, l)) = best_action {
        do_op(n, dir_step, board, plates, ops, p, k, d, l);
        return true;
    }

    false
}

// -------------------------------------------------------------
// 【核心新機能】雪だるま式チェーン回収（Snowball Snake Chain）
// -------------------------------------------------------------
fn snowball_chain_collect(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    color: usize,
    dist_table: &[Vec<usize>],
    ctx: &mut BfsContext,
) {
    let nest = plates[color];

    // 色 color を含むタワーの位置を収集
    let mut positions = Vec::with_capacity(32);
    for p in 0..n * n {
        if board.contains(p, color) {
            positions.push(p);
        }
    }

    if positions.is_empty() {
        return;
    }
    if positions.len() == 1 {
        move_smart_transfer(n, dir_step, max_ray, board, plates, ops, positions[0], nest, ctx);
        eject_nest_blockers_fast(n, dir_step, max_ray, board, plates, ops, nest, color);
        return;
    }

    // 巡回パスの生成: 巣から最も遠い点から開始
    let mut start_pos = positions[0];
    let mut max_dist = 0;
    for &p in &positions {
        let d = dist_table[p][nest];
        if d != usize::MAX && d > max_dist {
            max_dist = d;
            start_pos = p;
        }
    }

    let mut path = Vec::with_capacity(positions.len());
    let mut remaining = positions;
    let mut curr = start_pos;
    path.push(curr);
    remaining.retain(|&x| x != curr);

    while !remaining.is_empty() {
        let curr_h = board.len(curr);
        let mut best_idx = 0;
        let mut best_cost = isize::MAX;

        let r_c = curr / n;
        let c_c = curr % n;

        for (idx, &p) in remaining.iter().enumerate() {
            let r_p = p / n;
            let c_p = p % n;

            let mut direct_ray = false;
            let mut ray_dist = 0;
            if r_c == r_p {
                let d = if c_p > c_c { 3 } else { 2 };
                let dist = c_c.abs_diff(c_p);
                if max_ray[curr][d] >= dist {
                    direct_ray = true;
                    ray_dist = dist;
                }
            } else if c_c == c_p {
                let d = if r_p > r_c { 1 } else { 0 };
                let dist = r_c.abs_diff(r_p);
                if max_ray[curr][d] >= dist {
                    direct_ray = true;
                    ray_dist = dist;
                }
            }

            let dist = dist_table[curr][p];
            let mut cost = (dist as isize) * 10;

            if direct_ray && ray_dist == 1 {
                // 1マス隣: 1手で全員合体可能（超最高！）
                cost = -100;
            } else if direct_ray && ray_dist <= curr_h {
                // 直線ジャンプ射程内
                cost = (ray_dist as isize) * 2;
            }

            // 巣に近づく方向を好む
            let d_nest_curr = dist_table[curr][nest];
            let d_nest_p = dist_table[p][nest];
            if d_nest_curr != usize::MAX && d_nest_p != usize::MAX && d_nest_p <= d_nest_curr {
                cost -= 5;
            }

            if cost < best_cost {
                best_cost = cost;
                best_idx = idx;
            }
        }

        curr = remaining.remove(best_idx);
        path.push(curr);
    }

    // パスに沿って雪だるま式に回収・移動
    let mut snowball_pos = path[0];

    for &target in &path[1..] {
        if board.is_empty(target) {
            continue;
        }
        if board.is_empty(snowball_pos) {
            snowball_pos = target;
            continue;
        }
        if snowball_pos == target {
            continue;
        }

        let curr_h = board.len(snowball_pos);
        let tgt_h = board.len(target);
        let mask_combined = board.unique_color_mask(snowball_pos) | board.unique_color_mask(target);

        // タワーが満杯（高さ > 8、または 混載 > 3色）になる場合：巣へ配送してアンロード！
        if curr_h + tgt_h > 8 || mask_combined.count_ones() > 3 {
            move_smart_transfer(n, dir_step, max_ray, board, plates, ops, snowball_pos, nest, ctx);
            eject_nest_blockers_fast(n, dir_step, max_ray, board, plates, ops, nest, color);

            // 道中射出
            for p in 0..n * n {
                if board.len(p) >= 2 {
                    try_cross_color_ejection_fast(n, dir_step, max_ray, board, plates, ops, p, dist_table);
                }
            }

            snowball_pos = target;
            continue;
        }

        // snowball_pos から target へ移動（合体）
        let r_s = snowball_pos / n;
        let c_s = snowball_pos % n;
        let r_t = target / n;
        let c_t = target % n;

        // 隣接マスなら 1 手で全員合体！
        let is_adjacent = (r_s == r_t && c_s.abs_diff(c_t) == 1) || (c_s == c_t && r_s.abs_diff(r_t) == 1);
        if is_adjacent {
            let d = if r_s == r_t {
                if c_t > c_s { 3 } else { 2 }
            } else {
                if r_t > r_s { 1 } else { 0 }
            };
            do_op(n, dir_step, board, plates, ops, snowball_pos, 0, d, 1);
            snowball_pos = target;
        } else {
            // タワー合体歩行で迎えに行く
            if let Some(step_count) = find_path_merge_fast(dir_step, max_ray, board, plates, snowball_pos, target, curr_h, ctx) {
                let mut c_pos = snowball_pos;
                for idx in 0..step_count {
                    let d = ctx.path_tower[idx];
                    let np = (c_pos as isize + dir_step[d]) as usize;
                    if board.len(np) + board.len(c_pos) > 8 {
                        break;
                    }
                    do_op(n, dir_step, board, plates, ops, c_pos, 0, d, 1);
                    c_pos = np;

                    // 移動途中、頭に目的の色のスライムがあり巣と軸が合えば道中ダンク！
                    if board.len(c_pos) >= 2 {
                        try_cross_color_ejection_fast(n, dir_step, max_ray, board, plates, ops, c_pos, dist_table);
                    }

                    if c_pos == target || board.is_empty(c_pos) {
                        break;
                    }
                }
                snowball_pos = c_pos;
            } else {
                // BFSで見つからない場合（万一のフォールバック）
                move_smart_transfer(n, dir_step, max_ray, board, plates, ops, snowball_pos, target, ctx);
                snowball_pos = target;
            }
        }

        // 各合体ステップ後に道中射出をチェック
        if board.len(snowball_pos) >= 2 {
            try_cross_color_ejection_fast(n, dir_step, max_ray, board, plates, ops, snowball_pos, dist_table);
        }
    }

    // 最後に残った雪だるまタワーを巣へ配送
    if !board.is_empty(snowball_pos) {
        move_smart_transfer(n, dir_step, max_ray, board, plates, ops, snowball_pos, nest, ctx);
        eject_nest_blockers_fast(n, dir_step, max_ray, board, plates, ops, nest, color);
    }

    // 全盤面の道中射出
    for p in 0..n * n {
        if board.len(p) >= 2 {
            try_cross_color_ejection_fast(n, dir_step, max_ray, board, plates, ops, p, dist_table);
        }
    }
}

// -------------------------------------------------------------
// メインソルバー
// -------------------------------------------------------------
fn solve_advanced(
    n: usize,
    k: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    init_board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    color_order: &[usize],
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(400);

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
                        do_op(n, dir_step, &mut board, plates, &mut ops, p, 0, d, 1);
                    } else if dist_l > 1 && h >= dist_l {
                        let k_rem = dist_l - 1;
                        if board.len(target) + (h - k_rem) <= 8 {
                            do_op(n, dir_step, &mut board, plates, &mut ops, p, k_rem, d, dist_l);
                        }
                    }
                }
            }
        }
    }

    // Stage 2: 雪だるま式チェーン回収（盤面のスライムが減るまで反復）
    let mut outer = 0;
    while board.total_slimes() > 0 && outer < 25 {
        outer += 1;
        let mut progress = false;

        for &color in color_order {
            if board.count_color(color) == 0 {
                continue;
            }
            let prev_cnt = board.total_slimes();
            snowball_chain_collect(
                n,
                dir_step,
                max_ray,
                &mut board,
                plates,
                &mut ops,
                color,
                dist_table,
                ctx,
            );
            if board.total_slimes() < prev_cnt {
                progress = true;
            }
        }

        // 異色大ジャンプ射出
        for p in 0..n * n {
            if board.len(p) >= 2 {
                if try_cross_color_ejection_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, dist_table) {
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
                            do_op(n, dir_step, &mut board, plates, &mut ops, bp, k_rem, d, 1);
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

        // 単体強制移動（フォールバック）
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

    if board.total_slimes() > 0 {
        for p in 0..n * n {
            if !board.is_empty(p) {
                let h = board.len(p);
                let mut colors = Vec::new();
                let mut c_bits = board.colors[p];
                for _ in 0..h {
                    colors.push(c_bits & 0xF);
                    c_bits >>= 4;
                }
                eprintln!("REMAINING: p={} (r={}, c={}) h={} colors={:?} nest={:?}", p, p / n, p % n, h, colors, plates.iter().position(|&x| x == p));
            }
        }
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

    let mut ctx = BfsContext::new();

    // 初期の色順序: スライム数が多い順
    let mut current_order: Vec<usize> = (0..k).collect();
    current_order.sort_by_key(|&c| std::cmp::Reverse(init_board.count_color(c)));

    let (mut best_score, mut best_ops) = solve_advanced(
        n,
        k,
        &dir_step,
        &max_ray,
        &init_board,
        &plates,
        &dist_table,
        &current_order,
        &mut ctx,
    );

    let mut rng = XorShift::new(123456789);
    let mut iter_count = 0;

    // 時間が許す限り色順序を焼きなまし / 近傍探索
    let t_start = 10.0;
    let t_end = 0.5;
    let mut current_score = best_score;

    while start_time.elapsed() < total_time_limit {
        iter_count += 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let total = total_time_limit.as_secs_f64();
        let elapsed_ratio = (elapsed / total).min(1.0);
        let temperature = t_start * (1.0 - elapsed_ratio) + t_end * elapsed_ratio;

        let mut next_order = current_order.clone();
        let move_type = rng.gen_range(0, 100);
        if move_type < 40 {
            let i = rng.gen_range(0, k);
            let j = rng.gen_range(0, k);
            next_order.swap(i, j);
        } else if move_type < 70 {
            let from = rng.gen_range(0, k);
            let to = rng.gen_range(0, k);
            let val = next_order.remove(from);
            next_order.insert(to, val);
        } else {
            let mut i = rng.gen_range(0, k);
            let mut j = rng.gen_range(0, k);
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            next_order[i..=j].reverse();
        }

        let (cand_score, cand_ops) = solve_advanced(
            n,
            k,
            &dir_step,
            &max_ray,
            &init_board,
            &plates,
            &dist_table,
            &next_order,
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
            current_score = cand_score;

            if cand_score < best_score {
                best_score = cand_score;
                best_ops = cand_ops;
            }
        }
    }

    eprintln!(
        "STAT: iters={} best_score={}",
        iter_count, best_score
    );

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &best_ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
