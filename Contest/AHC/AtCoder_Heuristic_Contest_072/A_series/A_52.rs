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
    fn color_mask(&self, p: usize) -> u32 {
        let l = self.lens[p] as usize;
        let mut mask = 0u32;
        let mut c = self.colors[p];
        for _ in 0..l {
            mask |= 1 << (c & 0xF);
            c >>= 4;
        }
        mask
    }

    #[inline(always)]
    fn is_pure_color(&self, p: usize, color: usize) -> bool {
        let l = self.lens[p] as usize;
        if l == 0 {
            return false;
        }
        let mut c = self.colors[p];
        for _ in 0..l {
            if ((c & 0xF) as usize) != color {
                return false;
            }
            c >>= 4;
        }
        true
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
// 単体スライム用トランポリン探索（Step 44 の高速 BFS）
// -------------------------------------------------------------
fn find_path_single_trampoline_fast(
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &BitBoard,
    start: usize,
    target: usize,
    ctx: &mut BfsContext,
) -> Option<usize> {
    if start == target {
        return Some(0);
    }

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

// -------------------------------------------------------------
// タワー合体歩行 BFS（Step 15 の強力な合体許容 BFS）
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
// 巣を塞ぐ異色スライムの即時退避（エジェクション）
// -------------------------------------------------------------
fn eject_nest_blockers_fast(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    target_plate: usize,
    target_color: usize,
) -> bool {
    let mut changed = false;
    let mut loop_count = 0;

    while loop_count < 6 {
        loop_count += 1;
        let h = board.len(target_plate);
        if h == 0 {
            break;
        }

        let top_color = board.last(target_plate).unwrap();
        if top_color == target_color {
            board.clean_nest(target_plate, target_color);
            changed = true;
            continue;
        }

        let (top_len, _) = board.count_top_consecutive(target_plate);
        let k_rem = h - top_len;

        let mut best_d = None;
        let mut best_score = -100;

        for d in 0..4 {
            let ray = max_ray[target_plate][d];
            if ray < 1 {
                continue;
            }
            let np = (target_plate as isize + dir_step[d]) as usize;
            if board.len(np) + top_len <= 8 {
                let mut score = 0;
                if board.is_empty(np) {
                    score += 10;
                }
                if plates.contains(&np) {
                    score -= 5;
                }
                if score > best_score {
                    best_score = score;
                    best_d = Some(d);
                }
            }
        }

        if let Some(d) = best_d {
            let np = (target_plate as isize + dir_step[d]) as usize;
            ops.push(Op {
                r: target_plate / n,
                c: target_plate % n,
                k: k_rem,
                d: DIR_CHAR[d],
                l: 1,
            });

            board.transfer_and_reverse(target_plate, np, k_rem);

            for &chk in &[target_plate, np] {
                if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                    board.clean_nest(chk, pc);
                }
            }
            changed = true;
        } else {
            break;
        }
    }
    changed
}

// -------------------------------------------------------------
// 異色混合タワーからの大ジャンプ射出・デポ（Step 38）
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

    let (top_len, top_color) = board.count_top_consecutive(p);
    if top_len == h {
        return false;
    }

    let nest = plates[top_color];
    let k_rem = h - top_len;
    let max_l = (k_rem + 1).min(8);

    let mut best_choice = None;
    let mut best_score = isize::MIN;

    let cur_d = dist_table[p][nest];

    for d in 0..4 {
        let ray = max_ray[p][d];
        let limit = ray.min(max_l);
        if limit < 1 {
            continue;
        }

        let step = dir_step[d];
        let mut v = p as isize;

        for l in 1..=limit {
            v += step;
            let np = v as usize;

            if board.len(np) + top_len > 8 {
                continue;
            }

            let d_np = dist_table[np][nest];
            if d_np == usize::MAX {
                continue;
            }

            let mut score = (cur_d as isize - d_np as isize) * 10;

            if np == nest {
                score += 1000;
            } else {
                let r_np = np / n;
                let c_np = np % n;
                let r_nest = nest / n;
                let c_nest = nest % n;
                if r_np == r_nest || c_np == c_nest {
                    score += 5;
                }
                if board.is_empty(np) {
                    score += 2;
                }
                score += l as isize;
            }

            if score > best_score {
                best_score = score;
                best_choice = Some((k_rem, d, l, np));
            }
        }
    }

    if let Some((k, d, l, np)) = best_choice {
        if best_score > 0 {
            ops.push(Op {
                r: p / n,
                c: p % n,
                k,
                d: DIR_CHAR[d],
                l,
            });

            board.transfer_and_reverse(p, np, k);

            for &chk in &[p, np] {
                if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                    board.clean_nest(chk, pc);
                }
            }
            return true;
        }
    }

    false
}

// -------------------------------------------------------------
// マルチスライム一括射出（Bulk Direct Launch）
// -------------------------------------------------------------
fn try_bulk_direct_nest_launch(
    n: usize,
    _dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    p: usize,
    color: usize,
) -> bool {
    let target = plates[color];
    if p == target {
        return false;
    }

    let h = board.len(p);
    if h == 0 {
        return false;
    }

    let r_p = p / n;
    let c_p = p % n;
    let r_t = target / n;
    let c_t = target % n;

    let mut on_axis = None;
    if r_p == r_t {
        if c_t > c_p {
            on_axis = Some((3, c_t - c_p));
        } else if c_p > c_t {
            on_axis = Some((2, c_p - c_t));
        }
    } else if c_p == c_t {
        if r_t > r_p {
            on_axis = Some((1, r_t - r_p));
        } else if r_p > r_t {
            on_axis = Some((0, r_p - r_t));
        }
    }

    let Some((d, dist_l)) = on_axis else {
        return false;
    };

    if max_ray[p][d] < dist_l {
        return false;
    }

    let (top_len, top_color) = board.count_top_consecutive(p);
    if top_color != color {
        return false;
    }

    let mut best_k = None;

    if dist_l == 1 && top_len == h && board.len(target) + h <= 8 {
        best_k = Some(0);
    } else {
        let max_k = h - 1;
        let min_k = if dist_l > 1 { dist_l - 1 } else { 0 };

        for k in min_k..=max_k {
            let flying = h - k;
            if flying > top_len {
                continue;
            }
            if dist_l <= k + 1 {
                if board.len(target) + flying <= 8 {
                    best_k = Some(k);
                    break;
                }
            }
        }
    }

    let Some(k) = best_k else {
        return false;
    };

    ops.push(Op {
        r: r_p,
        c: c_p,
        k,
        d: DIR_CHAR[d],
        l: dist_l,
    });

    board.transfer_and_reverse(p, target, k);

    for &chk in &[p, target] {
        if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
            board.clean_nest(chk, pc);
        }
    }

    true
}

// -------------------------------------------------------------
// マルチスライム2段跳躍一括帰巣（Bulk Hop and Launch）
// -------------------------------------------------------------
fn try_bulk_hop_and_launch(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    p: usize,
    color: usize,
) -> bool {
    let target = plates[color];
    if p == target {
        return false;
    }

    let h = board.len(p);
    if h == 0 {
        return false;
    }

    let (top_len, top_color) = board.count_top_consecutive(p);
    if top_color != color {
        return false;
    }

    let r_t = target / n;
    let c_t = target % n;

    for d1 in 0..4 {
        let ray1 = max_ray[p][d1];
        if ray1 < 1 {
            continue;
        }

        let max_k1 = h - 1;
        let mut possible_k1 = Vec::new();
        for k1 in 0..=max_k1 {
            let flying = h - k1;
            if flying <= top_len {
                possible_k1.push(k1);
            }
        }

        for &k1 in &possible_k1 {
            let flying = h - k1;
            let max_l1 = (k1 + 1).min(ray1);

            for l1 in 1..=max_l1 {
                let mid_p = (p as isize + dir_step[d1] * l1 as isize) as usize;
                if mid_p == target {
                    continue;
                }

                if board.len(mid_p) + flying > 8 {
                    continue;
                }

                let r_m = mid_p / n;
                let c_m = mid_p % n;

                let mut on_axis_mid = None;
                if r_m == r_t {
                    if c_t > c_m {
                        on_axis_mid = Some((3, c_t - c_m));
                    } else if c_m > c_t {
                        on_axis_mid = Some((2, c_m - c_t));
                    }
                } else if c_m == c_t {
                    if r_t > r_m {
                        on_axis_mid = Some((1, r_t - r_m));
                    } else if r_m > r_t {
                        on_axis_mid = Some((0, r_m - r_t));
                    }
                }

                let Some((d2, dist_l2)) = on_axis_mid else {
                    continue;
                };

                if max_ray[mid_p][d2] < dist_l2 {
                    continue;
                }

                let h_mid_new = board.len(mid_p) + flying;
                let max_k2 = h_mid_new - 1;
                let min_k2 = if dist_l2 > 1 { dist_l2 - 1 } else { 0 };

                let mut best_k2 = None;
                for k2 in min_k2..=max_k2 {
                    let flying2 = h_mid_new - k2;
                    if flying2 <= flying {
                        if dist_l2 <= k2 + 1 && board.len(target) + flying2 <= 8 {
                            best_k2 = Some(k2);
                            break;
                        }
                    }
                }

                if let Some(k2) = best_k2 {
                    // Step 1
                    ops.push(Op {
                        r: p / n,
                        c: p % n,
                        k: k1,
                        d: DIR_CHAR[d1],
                        l: l1,
                    });
                    board.transfer_and_reverse(p, mid_p, k1);
                    for &chk in &[p, mid_p] {
                        if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                            board.clean_nest(chk, pc);
                        }
                    }

                    // Step 2
                    ops.push(Op {
                        r: r_m,
                        c: c_m,
                        k: k2,
                        d: DIR_CHAR[d2],
                        l: dist_l2,
                    });
                    board.transfer_and_reverse(mid_p, target, k2);
                    for &chk in &[mid_p, target] {
                        if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                            board.clean_nest(chk, pc);
                        }
                    }

                    return true;
                }
            }
        }
    }

    false
}

// -------------------------------------------------------------
// 統合版移動関数（move_smart_transfer）
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

    // 1. 高さ 1 の単体スライムは直接ジャンプ輸送へ
    if h == 1 {
        return move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, start, target, ctx);
    }

    let r_s = start / n;
    let c_s = start % n;
    let r_t = target / n;
    let c_t = target % n;

    // 2. (A) 直線大ジャンプ（途中のスライムタワーを飛び越え！）
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
    }

    // 2. (B) タワー合体歩行
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

    // 2. (C) 単体・分割トランポリン輸送（フォールバック）
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

// =============================================================
// Step 52 コア機能: 完全安全化された初期スペース創出
// =============================================================

// 1. 純粋同色単体スライムのみを安全に即時帰巣（安全チェック徹底）
fn clear_immediate_nest_sinks_safe(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    k_colors: usize,
) -> usize {
    let mut cleared_total = 0;
    for c in 0..k_colors {
        let target = plates[c];
        for d in 0..4 {
            let opp_d = match d {
                0 => 1,
                1 => 0,
                2 => 3,
                _ => 2,
            };
            if max_ray[target][opp_d] >= 1 {
                let p = (target as isize + dir_step[opp_d]) as usize;
                // 純粋に色 c のみで構成されており、巣の受け入れ可能高さ内である場合のみ！
                let h = board.len(p);
                if h > 0 && board.is_pure_color(p, c) && board.len(target) + h <= 8 {
                    ops.push(Op {
                        r: p / n,
                        c: p % n,
                        k: 0,
                        d: DIR_CHAR[d],
                        l: 1,
                    });
                    board.transfer_and_reverse(p, target, 0);
                    board.clean_nest(target, c);
                    cleared_total += h;
                }
            }
        }
    }
    cleared_total
}

// 2. 厳格3色制限＆スタック順序最適化マージ（安全版）
fn clear_space_merges_safe(
    n: usize,
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    dist_table: &[Vec<usize>],
    max_tower_h: usize,
    max_pairs: usize,
) -> usize {
    let mut merges = 0;
    while merges < max_pairs {
        let mut best_pair = None;
        let mut best_score = isize::MIN;

        for u in 0..n * n {
            let h_u = board.len(u);
            if h_u == 0 || plates.contains(&u) {
                continue;
            }

            let r_u = u / n;
            let c_u = u % n;
            let mask_u = board.color_mask(u);

            for d in 0..4 {
                let nr = r_u as isize + DIR[d].0;
                let nc = c_u as isize + DIR[d].1;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                    continue;
                }
                let v = nr as usize * n + nc as usize;
                let h_v = board.len(v);
                if h_v == 0 || plates.contains(&v) || h_u + h_v > max_tower_h || h_u + h_v > 8 {
                    continue;
                }

                // 3色制限チェック
                let mask_v = board.color_mask(v);
                let combined_mask = mask_u | mask_v;
                let unique_colors = combined_mask.count_ones() as usize;
                if unique_colors > 3 {
                    continue;
                }

                // スタック順序
                let u_bottom_color = (board.colors[u] & 0xF) as usize;
                let v_top_color = board.last(v).unwrap();

                let mut score = 0;
                if u_bottom_color == v_top_color {
                    score += 60;
                }
                if unique_colors == 1 {
                    score += 50;
                } else if unique_colors == 2 {
                    score += 20;
                } else {
                    score += 5;
                }

                let target_nest = plates[u_bottom_color];
                let d_u_nest = dist_table[u][target_nest];
                let d_v_nest = dist_table[v][target_nest];
                if d_v_nest <= d_u_nest {
                    score += 15;
                }
                score += (h_u + h_v) as isize;

                if score > best_score {
                    best_score = score;
                    best_pair = Some((u, v, d));
                }
            }
        }

        let Some((u, v, d)) = best_pair else {
            break;
        };

        ops.push(Op {
            r: u / n,
            c: u % n,
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
        merges += 1;
    }
    merges
}

// -------------------------------------------------------------
// メインソルバー（Step 52）
// -------------------------------------------------------------
fn solve_step52(
    n: usize,
    k: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    floor: &[bool],
    init_board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    color_order: &[usize],
    group_sizes: &[usize; 16],
    space_clearing_enabled: bool,
    max_clearing_pairs: usize,
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(400);

    let mut total_slimes = 0;
    for p in 0..n * n {
        total_slimes += board.len(p);
    }

    // ---------------------------------------------------------
    // Phase 0: 3色制限完全安全スペース創出
    // ---------------------------------------------------------
    if space_clearing_enabled && total_slimes >= 50 {
        clear_immediate_nest_sinks_safe(n, dir_step, max_ray, &mut board, plates, &mut ops, k);

        clear_space_merges_safe(
            n,
            &mut board,
            plates,
            &mut ops,
            dist_table,
            6,
            max_clearing_pairs,
        );
    }

    // ---------------------------------------------------------
    // Phase 1: 各色の効率的帰巣フェーズ
    // ---------------------------------------------------------
    let mut outer_loops = 0;
    loop {
        outer_loops += 1;
        if outer_loops > 25 {
            break;
        }

        let mut rem_slimes = 0;
        for p in 0..n * n {
            rem_slimes += board.len(p);
        }
        if rem_slimes == 0 {
            break;
        }

        let mut progress = false;

        for &color in color_order {
            let target_plate = plates[color];
            let group_max_size = group_sizes[color];

            let mut inner_loops = 0;
            loop {
                inner_loops += 1;
                if inner_loops > 50 {
                    break;
                }

                let mut positions = [0usize; 400];
                let mut pos_count = 0;
                for p in 0..n * n {
                    if board.last(p) == Some(color) {
                        positions[pos_count] = p;
                        pos_count += 1;
                    }
                }

                let total_rem = board.count_color(color);

                // 1. 直線一括射出
                let mut launched = false;
                for i in 0..pos_count {
                    let p = positions[i];
                    if try_bulk_direct_nest_launch(n, dir_step, max_ray, &mut board, plates, &mut ops, p, color) {
                        launched = true;
                        progress = true;
                        break;
                    }
                }
                if launched {
                    continue;
                }

                // 2. 2段中継跳躍
                let mut hopped = false;
                for i in 0..pos_count {
                    let p = positions[i];
                    let h_p = board.len(p);
                    if h_p == 1 && total_rem > 1 {
                        continue;
                    }
                    if try_bulk_hop_and_launch(n, dir_step, max_ray, &mut board, plates, &mut ops, p, color) {
                        hopped = true;
                        progress = true;
                        break;
                    }
                }
                if hopped {
                    continue;
                }

                if pos_count == 0 {
                    // 埋もれたスライムの掘り出し
                    let mut buried_p = None;
                    for p in 0..n * n {
                        if board.contains(p, color) {
                            buried_p = Some(p);
                            break;
                        }
                    }
                    if let Some(bp) = buried_p {
                        let (cnt, _) = board.count_top_consecutive(bp);
                        let h = board.len(bp);
                        let k_rem = h - cnt;

                        let mut ejected = false;
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

                                for &chk in &[bp, np] {
                                    if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                                        board.clean_nest(chk, pc);
                                    }
                                }
                                progress = true;
                                ejected = true;
                                break;
                            }
                        }
                        if ejected {
                            continue;
                        }
                    }
                    break;
                }

                progress = true;

                // 3. 中継・集約
                let mut best_dist = 0;
                let mut seed_p = positions[0];
                for i in 0..pos_count {
                    let p = positions[i];
                    let d = dist_table[p][target_plate];
                    if d > best_dist {
                        best_dist = d;
                        seed_p = p;
                    }
                }

                let mut sorted_pos = positions[..pos_count].to_vec();
                sorted_pos.sort_by_key(|&p| dist_table[p][seed_p]);
                let group_size = pos_count.min(group_max_size);
                let group = &sorted_pos[..group_size];

                let mut best_x = target_plate;
                let mut min_cost = usize::MAX;
                let r_target = target_plate / n;
                let c_target = target_plate % n;

                for x in 0..n * n {
                    if !floor[x] {
                        continue;
                    }
                    if !board.is_empty(x) && !group.contains(&x) && x != target_plate {
                        continue;
                    }
                    if dist_table[x][target_plate] == usize::MAX {
                        continue;
                    }

                    let mut cost = dist_table[x][target_plate];
                    let mut possible = true;
                    for &p in group {
                        if dist_table[p][x] == usize::MAX {
                            possible = false;
                            break;
                        }
                        cost += dist_table[p][x];
                    }

                    if possible {
                        let r_x = x / n;
                        let c_x = x % n;
                        if r_x == r_target || c_x == c_target {
                            cost = cost.saturating_sub(2);
                        }
                        if cost < min_cost {
                            min_cost = cost;
                            best_x = x;
                        }
                    }
                }

                let mut moved_any = false;
                for &p in group {
                    if p == best_x {
                        continue;
                    }
                    let success = move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, best_x, ctx);
                    if success {
                        moved_any = true;
                    } else if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, ctx) {
                        moved_any = true;
                    }
                }

                if best_x != target_plate && !board.is_empty(best_x) {
                    if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, ctx) {
                        moved_any = true;
                    }
                }

                eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, target_plate, color);

                for p in 0..n * n {
                    if board.len(p) >= 2 {
                        try_cross_color_ejection_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, dist_table);
                    }
                }

                if !moved_any {
                    break;
                }
            }
        }

        if !progress {
            for p in 0..n * n {
                if let Some(color) = board.last(p) {
                    let target_plate = plates[color];
                    move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, ctx);
                    break;
                }
            }
        }
    }

    // 最終回収フォールバック
    for _ in 0..30 {
        let mut any_rem = false;
        for p in 0..n * n {
            if let Some(color) = board.last(p) {
                any_rem = true;
                let target = plates[color];
                move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target, ctx);
            }
        }
        if !any_rem {
            break;
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
    let mut rng = XorShift::new(12345678901234567);

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

    let mut dist_desc = color_dists.clone();
    dist_desc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    order_candidates.push(dist_desc.iter().map(|&(c, _)| c).collect());

    let mut dist_asc = color_dists.clone();
    dist_asc.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    order_candidates.push(dist_asc.iter().map(|&(c, _)| c).collect());

    for _ in 0..4 {
        let mut shuf = (0..k).collect::<Vec<_>>();
        for i in (1..k).rev() {
            let j = rng.gen_range(0, i + 1);
            shuf.swap(i, j);
        }
        order_candidates.push(shuf);
    }

    let default_group_sizes = [6usize; 16];
    let mut best_score = usize::MAX;
    let mut best_ops = Vec::new();
    let mut best_order = order_candidates[0].clone();
    let mut best_space_clearing = true;
    let mut best_clearing_pairs = 20;

    for order in &order_candidates {
        for &sc in &[true, false] {
            let (score, ops) = solve_step52(
                n,
                k,
                &dir_step,
                &max_ray,
                &floor,
                &init_board,
                &plates,
                &dist_table,
                order,
                &default_group_sizes,
                sc,
                20,
                &mut ctx,
            );
            if score < best_score {
                best_score = score;
                best_ops = ops;
                best_order = order.clone();
                best_space_clearing = sc;
            }
        }
    }

    // ---------------------------------------------------------
    // Phase 2: 焼きなまし探索（SA）
    // ---------------------------------------------------------
    let mut current_order = best_order;
    let mut current_group_sizes = default_group_sizes;
    let mut current_space_clearing = best_space_clearing;
    let mut current_clearing_pairs = best_clearing_pairs;
    let mut current_score = best_score;

    let t_start: f64 = 8.0;
    let t_end: f64 = 0.5;
    let mut sa_iters = 0;

    while start_time.elapsed() < total_time_limit {
        sa_iters += 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let total = total_time_limit.as_secs_f64();
        let elapsed_ratio = (elapsed / total).min(1.0);
        let temperature = t_start * (1.0 - elapsed_ratio) + t_end * elapsed_ratio;

        let mut next_order = current_order.clone();
        let mut next_group_sizes = current_group_sizes;
        let mut next_space_clearing = current_space_clearing;
        let mut next_clearing_pairs = current_clearing_pairs;

        let r = rng.gen_range(0, 100);
        if r < 30 {
            let i = rng.gen_range(0, k);
            let j = rng.gen_range(0, k);
            next_order.swap(i, j);
        } else if r < 55 {
            let from = rng.gen_range(0, k);
            let to = rng.gen_range(0, k);
            let val = next_order.remove(from);
            next_order.insert(to, val);
        } else if r < 75 {
            let mut i = rng.gen_range(0, k);
            let mut j = rng.gen_range(0, k);
            if i > j { std::mem::swap(&mut i, &mut j); }
            next_order[i..=j].reverse();
        } else if r < 88 {
            let c = rng.gen_range(0, k);
            next_group_sizes[c] = rng.gen_range(4, 9);
        } else if r < 95 {
            next_clearing_pairs = rng.gen_range(10, 50);
        } else {
            next_space_clearing = !next_space_clearing;
        }

        let (cand_score, cand_ops) = solve_step52(
            n,
            k,
            &dir_step,
            &max_ray,
            &floor,
            &init_board,
            &plates,
            &dist_table,
            &next_order,
            &next_group_sizes,
            next_space_clearing,
            next_clearing_pairs,
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
            current_group_sizes = next_group_sizes;
            current_space_clearing = next_space_clearing;
            current_clearing_pairs = next_clearing_pairs;
            current_score = cand_score;

            if cand_score < best_score {
                best_score = cand_score;
                best_ops = cand_ops;
            }
        }
    }

    eprintln!(
        "STAT: sa_iters={} winner=Step52Safe final_score={}",
        sa_iters, best_score
    );

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &best_ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
