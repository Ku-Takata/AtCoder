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
    fn len(&self, p: usize) -> usize { self.lens[p] as usize }
    #[inline(always)]
    fn is_empty(&self, p: usize) -> bool { self.lens[p] == 0 }
    #[inline(always)]
    fn last(&self, p: usize) -> Option<usize> {
        let l = self.lens[p] as usize;
        if l == 0 { None } else { Some(((self.colors[p] >> ((l - 1) * 4)) & 0xF) as usize) }
    }
    #[inline(always)]
    fn contains(&self, p: usize, color: usize) -> bool {
        let l = self.lens[p] as usize;
        let mut c = self.colors[p];
        for _ in 0..l {
            if (c & 0xF) as usize == color { return true; }
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
        for _ in 0..block_len { rev = (rev << 4) | (temp & 0xF); temp >>= 4; }
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
            } else { break; }
        }
        cleaned
    }
    #[inline(always)]
    fn count_top_consecutive(&self, p: usize) -> (usize, usize) {
        let l = self.lens[p] as usize;
        if l == 0 { return (0, 0); }
        let top_color = ((self.colors[p] >> ((l - 1) * 4)) & 0xF) as usize;
        let mut count = 0;
        let mut shift = (l - 1) * 4;
        for _ in 0..l {
            if ((self.colors[p] >> shift) & 0xF) as usize == top_color {
                count += 1;
                if shift < 4 { break; }
                shift -= 4;
            } else { break; }
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
            for _ in 0..l { if ((c & 0xF) as usize) == color { count += 1; } c >>= 4; }
        }
        count
    }
    #[inline(always)]
    fn total_slimes(&self) -> usize {
        let mut t = 0; for p in 0..400 { t += self.lens[p] as usize; } t
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
            token: 0, visited: [0; 400], dist: [0; 400], prev: [0; 400],
            prev_action: [(0, 0, 0); 400], queue: [0; 400], actions: [(0, 0, 0, 0); 400],
            token_tower: 0, visited_tower: [0; 3600], dist_tower: [0; 3600],
            prev_tower: [(0, 0); 3600], queue_tower: [0; 3600], path_tower: [0; 400],
        }
    }
}

fn compute_max_ray(n: usize, floor: &[bool]) -> [[usize; 4]; 400] {
    let mut max_ray = [[0usize; 4]; 400];
    for r in 0..n { for c in 0..n {
        let u = r * n + c;
        if !floor[u] { continue; }
        let mut l = 0; while r >= l + 1 && floor[(r - 1 - l) * n + c] { l += 1; } max_ray[u][0] = l;
        let mut l = 0; while r + 1 + l < n && floor[(r + 1 + l) * n + c] { l += 1; } max_ray[u][1] = l;
        let mut l = 0; while c >= l + 1 && floor[r * n + (c - 1 - l)] { l += 1; } max_ray[u][2] = l;
        let mut l = 0; while c + 1 + l < n && floor[r * n + (c + 1 + l)] { l += 1; } max_ray[u][3] = l;
    }}
    max_ray
}

fn compute_all_pairs_dist(n: usize, floor: &[bool]) -> Vec<Vec<usize>> {
    let mut dist = vec![vec![usize::MAX; n * n]; n * n];
    let mut q = [0usize; 400];
    for start in 0..n * n {
        if !floor[start] { continue; }
        dist[start][start] = 0;
        let mut head = 0; let mut tail = 0;
        q[tail] = start; tail += 1;
        while head < tail {
            let u = q[head]; head += 1;
            let r = u / n; let c = u % n;
            for d in 0..4 {
                let nr = r as isize + DIR[d].0; let nc = c as isize + DIR[d].1;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize { continue; }
                let v = nr as usize * n + nc as usize;
                if floor[v] && dist[start][v] == usize::MAX {
                    dist[start][v] = dist[start][u] + 1; q[tail] = v; tail += 1;
                }
            }
        }
    }
    dist
}

#[inline(always)]
fn do_op(n: usize, dir_step: &[isize; 4], board: &mut BitBoard, plates: &[usize],
    ops: &mut Vec<Op>, p: usize, k: usize, d: usize, l: usize) -> usize {
    let next_p = (p as isize + dir_step[d] * l as isize) as usize;
    ops.push(Op { r: p / n, c: p % n, k, d: DIR_CHAR[d], l });
    board.transfer_and_reverse(p, next_p, k);
    for &chk in &[p, next_p] {
        if let Some(pc) = plates.iter().position(|&pos| pos == chk) { board.clean_nest(chk, pc); }
    }
    next_p
}

// 単体トランポリンBFS
fn find_path_single_trampoline(dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &BitBoard, start: usize, target: usize, ctx: &mut BfsContext) -> Option<usize> {
    if start == target { return Some(0); }
    ctx.token += 1; let token = ctx.token;
    ctx.visited[start] = token; ctx.dist[start] = 0;
    let mut head = 0; let mut tail = 0;
    ctx.queue[tail] = start; tail += 1;
    let mut reached = false;
    while head < tail {
        let u = ctx.queue[head]; head += 1;
        if u == target { reached = true; break; }
        let h = if u == start { board.len(start) } else { board.len(u) + 1 };
        if h == 0 { continue; }
        let k = h - 1; let max_l = k + 1;
        for d in 0..4 {
            let limit = max_l.min(max_ray[u][d]);
            let step = dir_step[d]; let mut v = u as isize;
            for l in 1..=limit {
                v += step; let next_p = v as usize;
                if board.len(next_p) + 1 > 8 { continue; }
                if ctx.visited[next_p] != token {
                    ctx.visited[next_p] = token;
                    ctx.dist[next_p] = ctx.dist[u] + 1;
                    ctx.prev[next_p] = u; ctx.prev_action[next_p] = (k, d, l);
                    ctx.queue[tail] = next_p; tail += 1;
                }
            }
        }
    }
    if !reached { return None; }
    let mut action_count = 0; let mut curr = target;
    while curr != start && action_count < 400 {
        let p = ctx.prev[curr]; let (k, d, l) = ctx.prev_action[curr];
        ctx.actions[action_count] = (p, k, d, l); action_count += 1; curr = p;
    }
    ctx.actions[..action_count].reverse();
    Some(action_count)
}

fn move_single_trampoline(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>,
    start: usize, target: usize, ctx: &mut BfsContext) -> bool {
    if start == target { return true; }
    let Some(action_count) = find_path_single_trampoline(dir_step, max_ray, board, start, target, ctx) else { return false; };
    if action_count == 0 { return true; }
    for idx in 0..action_count {
        let (p, k, d, l) = ctx.actions[idx];
        let h = board.len(p);
        if h == 0 || k >= h { return false; }
        let next_p = (p as isize + dir_step[d] * l as isize) as usize;
        if board.len(next_p) + (h - k) > 8 { return false; }
        let np = do_op(n, dir_step, board, plates, ops, p, k, d, l);
        if board.is_empty(np) { break; }
    }
    true
}

// タワー合体歩行BFS
fn find_path_merge(dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &BitBoard, start: usize, target: usize, start_h: usize,
    ctx: &mut BfsContext) -> Option<usize> {
    ctx.token_tower += 1; let token = ctx.token_tower;
    let start_idx = start * 9 + start_h;
    ctx.visited_tower[start_idx] = token; ctx.dist_tower[start_idx] = 0;
    let mut head = 0; let mut tail = 0;
    ctx.queue_tower[tail] = start_idx; tail += 1;
    let mut best_target_idx = None;
    while head < tail {
        let curr_idx = ctx.queue_tower[head]; head += 1;
        let u = curr_idx / 9; let h = curr_idx % 9;
        if u == target { best_target_idx = Some(curr_idx); break; }
        for d in 0..4 {
            if max_ray[u][d] < 1 { continue; }
            let v = (u as isize + dir_step[d]) as usize;
            let next_h = if v == target {
                if board.len(v) + h > 8 { continue; } h
            } else {
                let nh = h + board.len(v); if nh > 8 { continue; } nh
            };
            let next_idx = v * 9 + next_h;
            if ctx.visited_tower[next_idx] != token {
                ctx.visited_tower[next_idx] = token;
                ctx.dist_tower[next_idx] = ctx.dist_tower[curr_idx] + 1;
                ctx.prev_tower[next_idx] = (curr_idx, d);
                ctx.queue_tower[tail] = next_idx; tail += 1;
            }
        }
    }
    let Some(target_idx) = best_target_idx else { return None; };
    let mut step_count = 0; let mut curr_idx = target_idx;
    while curr_idx != start_idx && step_count < 400 {
        let (prev_idx, d) = ctx.prev_tower[curr_idx];
        ctx.path_tower[step_count] = d; step_count += 1; curr_idx = prev_idx;
    }
    ctx.path_tower[..step_count].reverse();
    Some(step_count)
}

// 異色射出
fn try_cross_color_ejection(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>, p: usize,
    dist_table: &[Vec<usize>]) -> bool {
    let h = board.len(p);
    if h < 2 { return false; }
    let (top_count, top_color) = board.count_top_consecutive(p);
    let k = h - top_count;
    if k == 0 { return false; }
    let max_l = (k + 1).min(8);
    let target_plate = plates[top_color];
    let curr_dist = dist_table[p][target_plate];
    let r_tgt = target_plate / n; let c_tgt = target_plate % n;
    let mut best_action = None; let mut best_gain = 0isize;
    for d in 0..4 {
        let limit = max_l.min(max_ray[p][d]);
        let step = dir_step[d]; let mut v = p as isize;
        for l in 1..=limit {
            v += step; let next_p = v as usize;
            if board.len(next_p) + top_count > 8 { continue; }
            if next_p == target_plate {
                let gain = 10000 + (l as isize) * 10;
                if gain > best_gain { best_gain = gain; best_action = Some((d, l)); }
                continue;
            }
            if board.len(next_p) > 0 && board.last(next_p) == Some(top_color) {
                let gain = 1000 + (l as isize) * 10;
                if gain > best_gain { best_gain = gain; best_action = Some((d, l)); }
                continue;
            }
            if l >= 2 {
                let next_dist = dist_table[next_p][target_plate];
                if next_dist < curr_dist {
                    let dist_saved = (curr_dist - next_dist) as isize;
                    let r_next = next_p / n; let c_next = next_p % n;
                    let mut on_axis_bonus: isize = 0;
                    if r_next == r_tgt || c_next == c_tgt {
                        on_axis_bonus = 15;
                        let (ad, adist) = if r_next == r_tgt {
                            if c_tgt > c_next { (3, c_tgt - c_next) } else { (2, c_next - c_tgt) }
                        } else {
                            if r_tgt > r_next { (1, r_tgt - r_next) } else { (0, r_next - r_tgt) }
                        };
                        if max_ray[next_p][ad] >= adist { on_axis_bonus = 55; if adist <= 4 { on_axis_bonus += 20; } }
                    }
                    let gain = dist_saved * 12 + (l as isize) * 4 + on_axis_bonus;
                    if gain > 10 && gain > best_gain { best_gain = gain; best_action = Some((d, l)); }
                }
            }
        }
    }
    if let Some((d, l)) = best_action { do_op(n, dir_step, board, plates, ops, p, k, d, l); return true; }
    false
}

// 統合移動関数（道中射出付き）
fn move_smart(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>,
    start: usize, target: usize, dist_table: &[Vec<usize>], ctx: &mut BfsContext) -> bool {
    if start == target || board.is_empty(start) { return true; }
    let h = board.len(start);
    if h == 1 { return move_single_trampoline(n, dir_step, max_ray, board, plates, ops, start, target, ctx); }

    let r_s = start / n; let c_s = start % n;
    let r_t = target / n; let c_t = target % n;

    // 直線直接ジャンプ
    let mut on_axis = None;
    if r_s == r_t {
        if c_t > c_s { on_axis = Some((3, c_t - c_s)); } else if c_s > c_t { on_axis = Some((2, c_s - c_t)); }
    } else if c_s == c_t {
        if r_t > r_s { on_axis = Some((1, r_t - r_s)); } else if r_s > r_t { on_axis = Some((0, r_s - r_t)); }
    }
    if let Some((d, dist_l)) = on_axis {
        if max_ray[start][d] >= dist_l {
            if dist_l == 1 && board.len(target) + h <= 8 {
                do_op(n, dir_step, board, plates, ops, start, 0, d, 1); return true;
            }
            if dist_l > 1 && h >= dist_l {
                let k = dist_l - 1;
                let mut base_has_target = false;
                if let Some(tc) = plates.iter().position(|&pos| pos == target) {
                    let mut c = board.colors[start];
                    for _ in 0..k { if ((c & 0xF) as usize) == tc { base_has_target = true; break; } c >>= 4; }
                }
                if !base_has_target {
                    let flying = h - k;
                    if board.len(target) + flying <= 8 {
                        do_op(n, dir_step, board, plates, ops, start, k, d, dist_l);
                        if board.is_empty(start) { return true; }
                    }
                }
            }
        }
    }

    // 合体歩行BFS + 道中射出
    let curr_h = board.len(start);
    if curr_h > 0 {
        if let Some(step_count) = find_path_merge(dir_step, max_ray, board, start, target, curr_h, ctx) {
            let mut curr = start;
            for idx in 0..step_count {
                if board.is_empty(curr) { break; }
                let d = ctx.path_tower[idx];
                do_op(n, dir_step, board, plates, ops, curr, 0, d, 1);
                let next_p = (curr as isize + dir_step[d]) as usize;
                // 道中射出
                if board.len(next_p) >= 2 {
                    try_cross_color_ejection(n, dir_step, max_ray, board, plates, ops, next_p, dist_table);
                }
                curr = next_p;
                if curr == target || board.is_empty(curr) { break; }
            }
            return true;
        }
    }

    // フォールバック
    let mut cnt = 0;
    while !board.is_empty(start) && cnt < 10 {
        if !move_single_trampoline(n, dir_step, max_ray, board, plates, ops, start, target, ctx) { break; }
        cnt += 1;
    }
    start == target || board.is_empty(start)
}

// 一括ダイレクト射出
fn try_direct_launch(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>, p: usize, color: usize) -> bool {
    let target = plates[color];
    if p == target { return false; }
    let r_p = p / n; let c_p = p % n; let r_t = target / n; let c_t = target % n;
    let (d, dist_l) = if r_p == r_t {
        if c_t > c_p { (3, c_t - c_p) } else if c_p > c_t { (2, c_p - c_t) } else { return false; }
    } else if c_p == c_t {
        if r_t > r_p { (1, r_t - r_p) } else if r_p > r_t { (0, r_p - r_t) } else { return false; }
    } else { return false; };
    if max_ray[p][d] < dist_l { return false; }
    let h = board.len(p); if dist_l > h { return false; }
    let k = dist_l - 1;
    if k > 0 { let mut c = board.colors[p]; for _ in 0..k { if ((c & 0xF) as usize) == color { return false; } c >>= 4; } }
    if board.len(target) + (h - k) > 8 { return false; }
    do_op(n, dir_step, board, plates, ops, p, k, d, dist_l); true
}

// 2段跳躍
fn try_hop_launch(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>, p: usize, color: usize) -> bool {
    let target = plates[color]; if p == target { return false; }
    let h_p = board.len(p); if h_p == 0 { return false; }
    let r_t = target / n; let c_t = target % n;
    for d1 in 0..4 {
        let limit1 = h_p.min(max_ray[p][d1]); let step1 = dir_step[d1]; let mut cm = p as isize;
        for l1 in 1..=limit1 {
            cm += step1; let m = cm as usize;
            if m == target || plates.contains(&m) { continue; }
            let k1 = l1 - 1; let fly1 = h_p - k1; let h_m = board.len(m);
            if h_m + fly1 > 8 { continue; }
            let r_m = m / n; let c_m = m % n;
            let (d2, dl2) = if r_m == r_t {
                if c_t > c_m { (3, c_t - c_m) } else if c_m > c_t { (2, c_m - c_t) } else { continue; }
            } else if c_m == c_t {
                if r_t > r_m { (1, r_t - r_m) } else if r_m > r_t { (0, r_m - r_t) } else { continue; }
            } else { continue; };
            if max_ray[m][d2] < dl2 { continue; }
            let ch = h_m + fly1; if dl2 > ch { continue; }
            let k2 = dl2 - 1; let fly2 = ch - k2;
            if board.len(target) + fly2 > 8 { continue; }
            do_op(n, dir_step, board, plates, ops, p, k1, d1, l1);
            do_op(n, dir_step, board, plates, ops, m, k2, d2, dl2);
            return true;
        }
    }
    false
}

// 巣ブロッカー排出（改良版）
fn eject_nest_blockers(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>,
    nest: usize, nest_color: usize, dist_table: &[Vec<usize>]) {
    let mut eject_count = 0;
    while !board.is_empty(nest) && eject_count < 10 {
        if board.last(nest) == Some(nest_color) {
            board.clean_nest(nest, nest_color);
            if board.is_empty(nest) { break; }
        }
        let (count, flying_color) = board.count_top_consecutive(nest);
        let h = board.len(nest); let k = h - count; let max_l = (k + 1).min(8);
        let target_for_flying = plates[flying_color];
        let r_tgt = target_for_flying / n; let c_tgt = target_for_flying % n;
        let curr_dist = dist_table[nest][target_for_flying];
        let mut best_action = None; let mut best_val = isize::MIN;
        for d in 0..4 {
            let limit = max_l.min(max_ray[nest][d]);
            let step = dir_step[d]; let mut v = nest as isize;
            for l in 1..=limit {
                v += step; let np = v as usize;
                if board.len(np) + count > 8 { continue; }
                if np == target_for_flying { let val = 10000 + (l as isize) * 10; if val > best_val { best_val = val; best_action = Some((d, l)); } continue; }
                if board.len(np) > 0 && board.last(np) == Some(flying_color) { let val = 1000 + (l as isize) * 10; if val > best_val { best_val = val; best_action = Some((d, l)); } continue; }
                let next_dist = dist_table[np][target_for_flying]; if next_dist == usize::MAX { continue; }
                let r_n = np / n; let c_n = np % n;
                let mut bonus: isize = 0;
                if r_n == r_tgt || c_n == c_tgt {
                    bonus = 15;
                    let (ad, adist) = if r_n == r_tgt { if c_tgt > c_n { (3, c_tgt - c_n) } else { (2, c_n - c_tgt) } }
                        else { if r_tgt > r_n { (1, r_tgt - r_n) } else { (0, r_n - r_tgt) } };
                    if max_ray[np][ad] >= adist { bonus = 50; if adist <= 4 { bonus += 20; } }
                }
                let dsaved = if curr_dist != usize::MAX && next_dist < curr_dist { (curr_dist - next_dist) as isize } else { 0 };
                let val = dsaved * 20 + (l as isize) * 5 + bonus;
                if val > best_val { best_val = val; best_action = Some((d, l)); }
            }
        }
        let Some((d, l)) = best_action else { break; };
        do_op(n, dir_step, board, plates, ops, nest, k, d, l);
        board.clean_nest(nest, nest_color);
        eject_count += 1;
    }
}

// 事前クラスタリング
fn pre_cluster_towers(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>,
    color: usize, dist_table: &[Vec<usize>], ctx: &mut BfsContext) {
    let target_plate = plates[color];
    let ts = board.total_slimes();
    let max_d = if ts >= 100 { 2 } else if ts >= 60 { 4 } else { 7 };
    for _ in 0..8 {
        let mut pos = [0usize; 400]; let mut pc = 0;
        for p in 0..n * n { if board.last(p) == Some(color) { pos[pc] = p; pc += 1; } }
        if pc < 2 { break; }
        let mut best = None; let mut mg = isize::MIN;
        for i in 0..pc { let p = pos[i]; let hp = board.len(p);
            let dp = dist_table[p][target_plate]; if dp == usize::MAX { continue; }
            for j in 0..pc { if i == j { continue; } let q = pos[j]; let hq = board.len(q);
                if hp + hq > 8 { continue; } let dq = dist_table[q][target_plate]; if dq == usize::MAX { continue; }
                let dpq = dist_table[p][q]; if dpq == usize::MAX || dpq > max_d { continue; }
                let mut g = (dp as isize) - (dpq as isize); if dq <= dp { g += 5; } g += hq as isize;
                if g > mg { mg = g; best = Some((p, q)); }
            }
        }
        let Some((p, q)) = best else { break; };
        if mg < 0 { break; }
        if !move_smart(n, dir_step, max_ray, board, plates, ops, p, q, dist_table, ctx) { break; }
    }
}

fn pre_cluster_multi(n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>,
    color: usize, dist_table: &[Vec<usize>], ctx: &mut BfsContext) {
    let ts = board.total_slimes();
    let max_d = if ts >= 100 { 2 } else if ts >= 60 { 3 } else { 5 };
    let mut my = [0usize; 32]; let mut mc = 0;
    for p in 0..n * n { if board.last(p) == Some(color) && board.len(p) > 0 && board.len(p) < 8 { my[mc] = p; mc += 1; if mc == 32 { break; } } }
    if mc == 0 { return; }
    let mut ot = [(0usize, 0usize); 64]; let mut oc = 0;
    for p in 0..n * n { let h = board.len(p); if h > 0 && h < 8 { ot[oc] = (p, h); oc += 1; if oc == 64 { break; } } }
    if oc < 2 { return; }
    for _ in 0..2 {
        let mut best = None; let mut mg = isize::MIN;
        for i in 0..mc { let p = my[i]; let hp = board.len(p); if hp == 0 || hp >= 8 { continue; }
            for j in 0..oc { let (q, _) = ot[j]; if p == q { continue; } let hq = board.len(q);
                if hq == 0 || hp + hq > 8 { continue; }
                let dpq = dist_table[p][q]; if dpq == usize::MAX || dpq > max_d { continue; }
                let ch = hp + hq; let mut g = (ch as isize) * 3 - (dpq as isize * 2); if ch >= 6 { g += 15; }
                if g > mg { mg = g; best = Some((p, q)); }
            }
        }
        let Some((p, q)) = best else { break; };
        if mg <= 0 { break; }
        if !move_smart(n, dir_step, max_ray, board, plates, ops, p, q, dist_table, ctx) { break; }
    }
}

// ============================================================================
// 核心改善: チェーン回収戦略
// 最も遠いスライムから出発し、巣に向かって途中のスライムを順次回収
// 独立移動の代わりに巡回ルートで輸送効率を大幅向上
// ============================================================================
fn chain_collect_and_deliver(
    n: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard, plates: &[usize], ops: &mut Vec<Op>,
    color: usize, dist_table: &[Vec<usize>], ctx: &mut BfsContext,
    chain_mode: u8,  // 0: farthest-first, 1: nearest-neighbor
) -> bool {
    let target = plates[color];

    let mut positions = Vec::new();
    for p in 0..n * n {
        if board.last(p) == Some(color) {
            positions.push(p);
        }
    }
    if positions.is_empty() { return false; }
    if positions.len() == 1 {
        return move_smart(n, dir_step, max_ray, board, plates, ops, positions[0], target, dist_table, ctx);
    }

    // チェーン順序の決定
    let chain: Vec<usize>;
    if chain_mode == 0 {
        // Farthest-first: 巣からの距離が遠い順
        let mut sorted = positions.clone();
        sorted.sort_by_key(|&p| std::cmp::Reverse(
            if dist_table[p][target] == usize::MAX { 0 } else { dist_table[p][target] }
        ));
        chain = sorted;
    } else {
        // Nearest-neighbor: 最遠点から始めて最近傍を順次選択
        let mut remaining = positions.clone();
        remaining.sort_by_key(|&p| std::cmp::Reverse(
            if dist_table[p][target] == usize::MAX { 0 } else { dist_table[p][target] }
        ));
        let mut result = Vec::with_capacity(remaining.len());
        let first = remaining.remove(0);
        result.push(first);
        let mut curr = first;
        while !remaining.is_empty() {
            let mut best_idx = 0;
            let mut best_d = usize::MAX;
            for (i, &p) in remaining.iter().enumerate() {
                let d = dist_table[curr][p];
                if d < best_d { best_d = d; best_idx = i; }
            }
            curr = remaining.remove(best_idx);
            result.push(curr);
        }
        chain = result;
    }

    // チェーン実行: 各位置を順に訪問して回収
    let mut curr_pos = chain[0];
    let mut batch_count = 0;

    for i in 1..chain.len() {
        let next = chain[i];
        if board.is_empty(curr_pos) || board.last(curr_pos) != Some(color) {
            curr_pos = next;
            continue;
        }

        let curr_h = board.len(curr_pos);
        let next_h = board.len(next);

        // タワーが満杯に近い場合、先に巣に届ける
        if curr_h >= 7 || (curr_h + next_h > 8 && curr_h >= 4) {
            move_smart(n, dir_step, max_ray, board, plates, ops, curr_pos, target, dist_table, ctx);
            eject_nest_blockers(n, dir_step, max_ray, board, plates, ops, target, color, dist_table);
            curr_pos = next;
            batch_count += 1;
            continue;
        }

        // 次のスライムに向かって歩く
        if board.len(next) > 0 && curr_h + next_h <= 8 {
            move_smart(n, dir_step, max_ray, board, plates, ops, curr_pos, next, dist_table, ctx);
            curr_pos = next;
        } else if board.len(next) > 0 {
            // 合体できない場合、先に巣に配送
            move_smart(n, dir_step, max_ray, board, plates, ops, curr_pos, target, dist_table, ctx);
            eject_nest_blockers(n, dir_step, max_ray, board, plates, ops, target, color, dist_table);
            curr_pos = next;
            batch_count += 1;
        } else {
            // nextは既に空（途中で帰巣された）
            // curr_posをそのまま維持
        }
    }

    // 最後のバッチを巣に配送
    if !board.is_empty(curr_pos) && board.last(curr_pos) == Some(color) {
        move_smart(n, dir_step, max_ray, board, plates, ops, curr_pos, target, dist_table, ctx);
    }
    eject_nest_blockers(n, dir_step, max_ray, board, plates, ops, target, color, dist_table);

    // 異色射出
    for p in 0..n * n {
        if board.len(p) >= 2 { try_cross_color_ejection(n, dir_step, max_ray, board, plates, ops, p, dist_table); }
    }

    true
}

// ============================================================================
// メインソルバー
// ============================================================================
fn solve(
    n: usize, k: usize, dir_step: &[isize; 4], max_ray: &[[usize; 4]; 400],
    floor: &[bool], init_board: &BitBoard, plates: &[usize], dist_table: &[Vec<usize>],
    color_order: &[usize], group_sizes: &[usize; 16], strategies: &[u8; 16],
    stage1_modes: &[u8; 16], ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(400);

    // Stage 1: 事前クラスタリング
    for &color in color_order {
        match stage1_modes[color] {
            1 => {
                let mut pos = [0usize; 400]; let mut pc = 0;
                for p in 0..n * n { if board.last(p) == Some(color) { pos[pc] = p; pc += 1; } }
                if pc >= 2 {
                    for i in 0..pc { let p = pos[i]; if board.last(p) != Some(color) { continue; } let hp = board.len(p);
                        for j in (i+1)..pc { let q = pos[j]; if board.last(q) != Some(color) { continue; } let hq = board.len(q);
                            if hp + hq <= 8 { let d = dist_table[p][q]; if d <= 2 { if move_smart(n, dir_step, max_ray, &mut board, plates, &mut ops, p, q, dist_table, ctx) { break; } } }
                        }
                    }
                }
            }
            2 => { pre_cluster_towers(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx); }
            3 => {
                pre_cluster_towers(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx);
                pre_cluster_multi(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx);
            }
            _ => {}
        }
    }

    // Stage 2: 帰巣フェーズ
    let mut outer = 0;
    loop {
        outer += 1; if outer > 20 { break; }
        if board.total_slimes() == 0 { break; }
        let mut progress = false;

        for &color in color_order {
            let target_plate = plates[color];
            let group_max = group_sizes[color];
            let strat = strategies[color];

            let mut inner = 0;
            loop {
                inner += 1; if inner > 50 { break; }

                let mut pos = [0usize; 400]; let mut pc = 0;
                for p in 0..n * n { if board.last(p) == Some(color) { pos[pc] = p; pc += 1; } }
                let total_rem = board.count_color(color);

                // ダイレクト射出
                let mut launched = false;
                for i in 0..pc {
                    if try_direct_launch(n, dir_step, max_ray, &mut board, plates, &mut ops, pos[i], color) { launched = true; progress = true; break; }
                }
                if launched { continue; }

                // 2段跳躍
                let mut hopped = false;
                for i in 0..pc {
                    if board.len(pos[i]) == 1 && total_rem > 1 { continue; }
                    if try_hop_launch(n, dir_step, max_ray, &mut board, plates, &mut ops, pos[i], color) { hopped = true; progress = true; break; }
                }
                if hopped { continue; }

                // 埋もれたスライム掘り出し
                if pc == 0 {
                    let mut bp = None;
                    for p in 0..n * n { if board.contains(p, color) { bp = Some(p); break; } }
                    if let Some(b) = bp {
                        let (cnt, _) = board.count_top_consecutive(b);
                        let h = board.len(b); let kr = h - cnt;
                        let mut ej = false;
                        for d in 0..4 {
                            if max_ray[b][d] < 1 { continue; }
                            let np = (b as isize + dir_step[d]) as usize;
                            if board.len(np) + cnt <= 8 { do_op(n, dir_step, &mut board, plates, &mut ops, b, kr, d, 1); progress = true; ej = true; break; }
                        }
                        if ej { continue; }
                    }
                    break;
                }

                progress = true;

                // ======= 戦略分岐 =======
                if strat == 3 {
                    // チェーン回収戦略（farthest-first）
                    chain_collect_and_deliver(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx, 0);
                    break; // このcolorのinner loopを抜ける（チェーンが全部処理する）
                } else if strat == 4 {
                    // チェーン回収戦略（nearest-neighbor）
                    chain_collect_and_deliver(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx, 1);
                    break;
                }

                // strat 0 or 2: 従来のミーティングポイント戦略
                let mut best_dist = 0; let mut seed_p = pos[0];
                for i in 0..pc { let d = dist_table[pos[i]][target_plate]; if d > best_dist { best_dist = d; seed_p = pos[i]; } }
                let mut sorted_pos = pos[..pc].to_vec();
                sorted_pos.sort_by_key(|&p| dist_table[p][seed_p]);
                let gs = pc.min(group_max);
                let group = &sorted_pos[..gs];

                let best_x = if strat == 2 { target_plate } else {
                    let mut min_cost = usize::MAX; let mut best = target_plate;
                    let rt = target_plate / n; let ct = target_plate % n;
                    for x in 0..n * n {
                        if !floor[x] { continue; }
                        if !board.is_empty(x) && !group.contains(&x) && x != target_plate { continue; }
                        if dist_table[x][target_plate] == usize::MAX { continue; }
                        let mut cost = dist_table[x][target_plate]; let mut ok = true;
                        for &p in group { if dist_table[p][x] == usize::MAX { ok = false; break; } cost += dist_table[p][x]; }
                        if ok {
                            let rx = x / n; let cx = x % n;
                            if rx == rt || cx == ct { cost = cost.saturating_sub(1); }
                            if cost < min_cost { min_cost = cost; best = x; }
                        }
                    }
                    best
                };

                let mut moved_any = false;
                for &p in group {
                    if p == best_x { continue; }
                    if move_smart(n, dir_step, max_ray, &mut board, plates, &mut ops, p, best_x, dist_table, ctx) { moved_any = true; }
                    else if move_smart(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, dist_table, ctx) { moved_any = true; }
                }
                if best_x != target_plate && !board.is_empty(best_x) {
                    if move_smart(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, dist_table, ctx) { moved_any = true; }
                }
                eject_nest_blockers(n, dir_step, max_ray, &mut board, plates, &mut ops, target_plate, color, dist_table);
                for p in 0..n * n { if board.len(p) >= 2 { try_cross_color_ejection(n, dir_step, max_ray, &mut board, plates, &mut ops, p, dist_table); } }
                if !moved_any { break; }
            }
        }

        if !progress {
            for p in 0..n * n {
                if let Some(c) = board.last(p) {
                    move_smart(n, dir_step, max_ray, &mut board, plates, &mut ops, p, plates[c], dist_table, ctx);
                    break;
                }
            }
        }
    }

    let unreturned = board.total_slimes();
    (ops.len() + unreturned * 100_000, ops)
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
                '#' => floor[p] = false, '.' => {}
                'a'..='z' => { init_stacks[p].push((ch as u8 - b'a') as usize); }
                'A'..='Z' => { plates[(ch as u8 - b'A') as usize] = p; }
                _ => {}
            }
        }
    }

    let dir_step: [isize; 4] = [-(n as isize), n as isize, -1, 1];
    let max_ray = compute_max_ray(n, &floor);
    let dist_table = compute_all_pairs_dist(n, &floor);
    let init_board = BitBoard::new(n, &init_stacks);
    let mut ctx = Box::new(BfsContext::new());
    let mut rng = XorShift::new(20260928);

    let mut best_score = usize::MAX;
    let mut best_ops = Vec::new();
    let default_group_sizes = [6usize; 16];
    let default_strategies = [0u8; 16];
    let default_stage1_modes = [2u8; 16];

    // 初期スクリーニング
    let mut order_candidates = Vec::new();
    order_candidates.push((0..k).collect::<Vec<_>>());
    let mut rev = (0..k).collect::<Vec<_>>(); rev.reverse(); order_candidates.push(rev);
    let mut cdists: Vec<(usize, f64)> = Vec::new();
    for c in 0..k {
        let tgt = plates[c]; let mut ds = 0usize; let mut cnt = 0usize;
        for p in 0..n * n { if init_board.contains(p, c) { let d = dist_table[p][tgt]; if d != usize::MAX { ds += d; cnt += 1; } } }
        cdists.push((c, if cnt > 0 { ds as f64 / cnt as f64 } else { 0.0 }));
    }
    let mut dd = cdists.clone(); dd.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    order_candidates.push(dd.iter().map(|&(c, _)| c).collect());
    let mut da = cdists.clone(); da.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    order_candidates.push(da.iter().map(|&(c, _)| c).collect());
    let mut cc: Vec<(usize, usize)> = (0..k).map(|c| (c, init_board.count_color(c))).collect();
    cc.sort_by_key(|&(_, cnt)| std::cmp::Reverse(cnt));
    order_candidates.push(cc.iter().map(|&(c, _)| c).collect());
    cc.sort_by_key(|&(_, cnt)| cnt);
    order_candidates.push(cc.iter().map(|&(c, _)| c).collect());
    for _ in 0..6 { let mut s = (0..k).collect::<Vec<_>>(); for i in (1..k).rev() { let j = rng.gen_range(0, i + 1); s.swap(i, j); } order_candidates.push(s); }

    let mut best_order = (0..k).collect::<Vec<_>>();
    let mut best_stage1 = default_stage1_modes;

    for order in &order_candidates {
        let (score, ops) = solve(n, k, &dir_step, &max_ray, &floor, &init_board, &plates, &dist_table,
            order, &default_group_sizes, &default_strategies, &default_stage1_modes, &mut ctx);
        if score < best_score { best_score = score; best_order = order.clone(); best_ops = ops; best_stage1 = default_stage1_modes; }
    }

    // チェーン戦略もスクリーニング
    for &chain_strat in &[3u8, 4u8] {
        let strats = [chain_strat; 16];
        let (score, ops) = solve(n, k, &dir_step, &max_ray, &floor, &init_board, &plates, &dist_table,
            &best_order, &default_group_sizes, &strats, &default_stage1_modes, &mut ctx);
        if score < best_score { best_score = score; best_ops = ops; }
    }

    // Courier Solo
    {
        let mut ms = [0u8; 16]; ms[best_order[0]] = 2;
        let (score, ops) = solve(n, k, &dir_step, &max_ray, &floor, &init_board, &plates, &dist_table,
            &best_order, &default_group_sizes, &default_strategies, &ms, &mut ctx);
        if score < best_score { best_score = score; best_ops = ops; best_stage1 = ms; }
    }

    // SA
    let mut cur_order = best_order.clone();
    let mut cur_gs = default_group_sizes;
    let mut cur_strats = default_strategies;
    let mut cur_s1 = best_stage1;
    let mut cur_score = best_score;
    let mut sa_best_score = best_score;
    let mut sa_best_ops = best_ops;
    let t_start: f64 = 8.0; let t_end: f64 = 0.5;
    let mut sa_iters = 0;

    while start_time.elapsed() < total_time_limit {
        sa_iters += 1;
        let ratio = (start_time.elapsed().as_secs_f64() / total_time_limit.as_secs_f64()).min(1.0);
        let temp = t_start * (1.0 - ratio) + t_end * ratio;

        let mut no = cur_order.clone();
        let mut ngs = cur_gs;
        let mut nst = cur_strats;
        let mut ns1 = cur_s1;

        let mt = rng.gen_range(0, 100);
        if mt < 15 { let i = rng.gen_range(0, k); let j = rng.gen_range(0, k); no.swap(i, j); }
        else if mt < 30 { let f = rng.gen_range(0, k); let t = rng.gen_range(0, k); let v = no.remove(f); no.insert(t, v); }
        else if mt < 42 { let mut i = rng.gen_range(0, k); let mut j = rng.gen_range(0, k); if i > j { std::mem::swap(&mut i, &mut j); } no[i..=j].reverse(); }
        else if mt < 55 { let c = rng.gen_range(0, k); ngs[c] = rng.gen_range(3, 9); }
        else if mt < 72 {
            // 戦略変更（0-4の5種類！）
            let c = rng.gen_range(0, k); nst[c] = rng.gen_range(0, 5) as u8;
        }
        else if mt < 87 { let c = rng.gen_range(0, k); ns1[c] = rng.gen_range(0, 4) as u8; }
        else {
            let sub = rng.gen_range(0, 4);
            match sub {
                0 => { let lead = no[0]; ns1 = [0u8; 16]; ns1[lead] = 2; }
                1 => { ns1 = [2u8; 16]; }
                2 => { nst = [3u8; 16]; } // 全色チェーン
                _ => { nst = [0u8; 16]; } // 全色ミーティングポイント
            }
        }

        let (cs, co) = solve(n, k, &dir_step, &max_ray, &floor, &init_board, &plates, &dist_table,
            &no, &ngs, &nst, &ns1, &mut ctx);

        let delta = cs as f64 - cur_score as f64;
        if delta <= 0.0 || rng.gen_f64() < (-delta / temp).exp() {
            cur_order = no; cur_gs = ngs; cur_strats = nst; cur_s1 = ns1; cur_score = cs;
            if cs < sa_best_score { sa_best_score = cs; sa_best_ops = co; }
        }
    }

    eprintln!("STAT: sa_iters={} final_score={}", sa_iters, sa_best_score);
    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &sa_best_ops { writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap(); }
}
