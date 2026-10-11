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
    fn clean_nest(&mut self, p: usize, plate_color: usize) {
        while self.lens[p] > 0 {
            let shift = (self.lens[p] as usize - 1) * 4;
            if ((self.colors[p] >> shift) & 0xF) as usize == plate_color {
                self.lens[p] -= 1;
                let new_l = self.lens[p] as usize;
                self.colors[p] &= if new_l == 0 { 0 } else { (1u32 << (new_l * 4)) - 1 };
            } else {
                break;
            }
        }
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
}

struct BfsContext {
    token: usize,
    visited: [usize; 400],
    dist: [usize; 400],
    prev: [usize; 400],
    prev_action: [(usize, usize, usize); 400],
    queue: [usize; 400],
    actions: [(usize, usize, usize, usize); 400],

    token_tower: usize,
    visited_tower: [usize; 3600],
    dist_tower: [usize; 3600],
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

fn find_path_merge_fast(
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &BitBoard,
    start: usize,
    target: usize,
    start_h: usize,
    ctx: &mut BfsContext,
) -> Option<usize> {
    ctx.token_tower += 1;
    let token = ctx.token_tower;

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

        for d in 0..4 {
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

fn move_tower_merge_fast(
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
    let tower_h = board.len(start);
    if tower_h == 0 || start == target {
        return true;
    }
    let Some(step_count) = find_path_merge_fast(dir_step, max_ray, board, start, target, tower_h, ctx) else {
        return false;
    };

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
        if curr == target {
            break;
        }
    }
    true
}

fn try_direct_nest_launch_fast(
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

    let r_p = p / n;
    let c_p = p % n;
    let r_t = target / n;
    let c_t = target % n;

    let (d, dist_l) = if r_p == r_t {
        if c_t > c_p {
            (3, c_t - c_p)
        } else {
            (2, c_p - c_t)
        }
    } else if c_p == c_t {
        if r_t > r_p {
            (1, r_t - r_p)
        } else {
            (0, r_p - r_t)
        }
    } else {
        return false;
    };

    if max_ray[p][d] < dist_l {
        return false;
    }

    let h = board.len(p);
    if dist_l > h {
        return false;
    }

    let k = dist_l - 1;
    if board.len(target) + (h - k) > 8 {
        return false;
    }

    ops.push(Op {
        r: r_p,
        c: c_p,
        k,
        d: DIR_CHAR[d],
        l: dist_l,
    });

    board.transfer_and_reverse(p, target, k);

    for &chk in &[p, target] {
        if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
            board.clean_nest(chk, plate_color);
        }
    }

    true
}

fn try_hop_and_launch_fast(
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

    let h_p = board.len(p);
    if h_p == 0 {
        return false;
    }

    let r_t = target / n;
    let c_t = target % n;

    for d1 in 0..4 {
        let limit1 = h_p.min(max_ray[p][d1]);
        let step1 = dir_step[d1];
        let mut curr_m = p as isize;

        for l1 in 1..=limit1 {
            curr_m += step1;
            let m = curr_m as usize;
            if m == target {
                continue;
            }

            let k1 = l1 - 1;
            let flying_b1 = h_p - k1;
            let h_m = board.len(m);
            if h_m + flying_b1 > 8 {
                continue;
            }

            let r_m = m / n;
            let c_m = m % n;

            let (d2, dist_l2) = if r_m == r_t {
                if c_t > c_m {
                    (3, c_t - c_m)
                } else {
                    (2, c_m - c_t)
                }
            } else if c_m == c_t {
                if r_t > r_m {
                    (1, r_t - r_m)
                } else {
                    (0, r_m - r_t)
                }
            } else {
                continue;
            };

            if max_ray[m][d2] < dist_l2 {
                continue;
            }

            let combined_h = h_m + flying_b1;
            if dist_l2 > combined_h {
                continue;
            }

            let k2 = dist_l2 - 1;
            let flying_b2 = combined_h - k2;
            if board.len(target) + flying_b2 > 8 {
                continue;
            }

            ops.push(Op {
                r: p / n,
                c: p % n,
                k: k1,
                d: DIR_CHAR[d1],
                l: l1,
            });
            board.transfer_and_reverse(p, m, k1);

            for &chk in &[p, m] {
                if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
                    board.clean_nest(chk, plate_color);
                }
            }

            ops.push(Op {
                r: r_m,
                c: c_m,
                k: k2,
                d: DIR_CHAR[d2],
                l: dist_l2,
            });
            board.transfer_and_reverse(m, target, k2);

            for &chk in &[m, target] {
                if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
                    board.clean_nest(chk, plate_color);
                }
            }

            return true;
        }
    }

    false
}

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
            board.pop(nest);
            continue;
        }

        let (count, _) = board.count_top_consecutive(nest);
        let h = board.len(nest);
        let k = h - count;
        let max_l = k + 1;

        let mut best_action = None;
        let mut min_load = usize::MAX;

        for d in 0..4 {
            let limit = max_l.min(max_ray[nest][d]);
            let step = dir_step[d];
            let mut v = nest as isize;

            for l in 1..=limit {
                v += step;
                let next_p = v as usize;
                if board.len(next_p) + count <= 8 {
                    if board.len(next_p) < min_load {
                        min_load = board.len(next_p);
                        best_action = Some((d, l, next_p));
                    }
                }
            }
        }

        let Some((d, l, next_p)) = best_action else {
            break;
        };

        ops.push(Op {
            r: nest / n,
            c: nest % n,
            k,
            d: DIR_CHAR[d],
            l,
        });

        board.transfer_and_reverse(nest, next_p, k);

        for &p in &[nest, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                board.clean_nest(p, plate_color);
            }
        }

        eject_count += 1;
    }
}

// -------------------------------------------------------------
// 新規：Stage 1 事前クラスタリング関数群
// -------------------------------------------------------------
// モード1: 局所ペアマージ（近接同色スライム同士を合体させ、曲がり角を残しつつ高さを底上げ）
fn apply_local_pair_merge(
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
    let mut positions = [0usize; 400];
    let mut pos_count = 0;
    for p in 0..n * n {
        if board.last(p) == Some(color) {
            positions[pos_count] = p;
            pos_count += 1;
        }
    }

    if pos_count < 2 {
        return;
    }

    for i in 0..pos_count {
        let p = positions[i];
        if board.last(p) != Some(color) { continue; }
        let h_p = board.len(p);

        for j in (i + 1)..pos_count {
            let q = positions[j];
            if board.last(q) != Some(color) { continue; }
            let h_q = board.len(q);

            if h_p + h_q <= 8 {
                let d = dist_table[p][q];
                if d == 1 || d == 2 {
                    if move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, p, q, ctx) {
                        break;
                    }
                }
            }
        }
    }
}

// モード2: 中間ハブ集約（巣と反対側のスライム群を中間位置に集約してタワー化）
fn apply_hub_staging(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    floor: &[bool],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    color: usize,
    group_max_size: usize,
    dist_table: &[Vec<usize>],
    ctx: &mut BfsContext,
) {
    let target_plate = plates[color];
    let mut positions = [0usize; 400];
    let mut pos_count = 0;
    for p in 0..n * n {
        if board.last(p) == Some(color) {
            positions[pos_count] = p;
            pos_count += 1;
        }
    }

    if pos_count < 2 {
        return;
    }

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

    let mut best_hub = seed_p;
    let mut min_cost = usize::MAX;

    for x in 0..n * n {
        if !floor[x] || x == target_plate { continue; }
        if !board.is_empty(x) && !group.contains(&x) { continue; }

        let mut cost = 0;
        let mut possible = true;
        for &p in group {
            let d = dist_table[p][x];
            if d == usize::MAX {
                possible = false;
                break;
            }
            cost += d;
        }

        if possible && cost < min_cost {
            min_cost = cost;
            best_hub = x;
        }
    }

    for &p in group {
        if p == best_hub { continue; }
        move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, p, best_hub, ctx);
    }
}

// -------------------------------------------------------------
// フェーズ 2 用: Stage 1 + Stage 2 完全統合シミュレーション関数
// -------------------------------------------------------------
fn solve_with_stages(
    n: usize,
    _k: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    floor: &[bool],
    init_board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    color_order: &[usize],
    group_sizes: &[usize; 16],
    strategies: &[u8; 16],
    stage1_modes: &[u8; 16],
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(300);

    // =========================================================
    // Stage 1: 事前クラスタリングフェーズ
    // スライムを消去せず、局所タワー化してジャンプ力と踏み台を確保
    // =========================================================
    for &color in color_order {
        match stage1_modes[color] {
            1 => {
                apply_local_pair_merge(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx);
            }
            2 => {
                apply_hub_staging(n, dir_step, max_ray, floor, &mut board, plates, &mut ops, color, group_sizes[color], dist_table, ctx);
            }
            _ => {}
        }
    }

    // =========================================================
    // Stage 2: 帰巣フェーズ
    // 巨大なジャンプ力を得たタワー群が、残るタワーを踏み台にして次々と巣へ帰巣
    // =========================================================
    let mut outer_loops = 0;
    loop {
        outer_loops += 1;
        if outer_loops > 20 {
            break;
        }

        let mut total_slimes = 0;
        for p in 0..n * n {
            total_slimes += board.len(p);
        }
        if total_slimes == 0 {
            break;
        }

        let mut progress = false;

        for &color in color_order {
            let target_plate = plates[color];
            let group_max_size = group_sizes[color];
            let strat = strategies[color];

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

                // 1. 1段ダイレクト射出
                let mut launched = false;
                for i in 0..pos_count {
                    let p = positions[i];
                    if try_direct_nest_launch_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, color) {
                        launched = true;
                        progress = true;
                        break;
                    }
                }
                if launched {
                    continue;
                }

                // 2. 2段トランポリン跳躍コンボ（Hop & Shot）
                let mut hopped = false;
                for i in 0..pos_count {
                    let p = positions[i];
                    if try_hop_and_launch_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, color) {
                        hopped = true;
                        progress = true;
                        break;
                    }
                }
                if hopped {
                    continue;
                }

                if pos_count == 0 {
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

                if strat == 2 {
                    best_x = target_plate;
                } else {
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
                            if strat == 1 {
                                let r_x = x / n;
                                let c_x = x % n;
                                if r_x == r_target || c_x == c_target {
                                    cost = cost.saturating_sub(4);
                                }
                            }
                            if cost < min_cost {
                                min_cost = cost;
                                best_x = x;
                            }
                        }
                    }
                }

                for &p in group {
                    if p == best_x {
                        continue;
                    }
                    let success = move_single_trampoline_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, best_x, ctx);
                    if !success {
                        move_single_trampoline_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, ctx);
                    }
                }

                if best_x != target_plate && !board.is_empty(best_x) {
                    let success = move_tower_merge_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, ctx);
                    if !success {
                        let mut fallback_count = 0;
                        while !board.is_empty(best_x) && fallback_count < 10 {
                            let s = move_single_trampoline_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, ctx);
                            if !s { break; }
                            fallback_count += 1;
                        }
                    }
                }

                eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, target_plate, color);
            }
        }

        if !progress {
            for p in 0..n * n {
                if let Some(color) = board.last(p) {
                    let target_plate = plates[color];
                    move_single_trampoline_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, ctx);
                    break;
                }
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

// -------------------------------------------------------------
// フェーズ 1 用: ビームサーチ関数群
// -------------------------------------------------------------
fn find_candidate_aggregation_points(
    n: usize,
    floor: &[bool],
    board: &BitBoard,
    target_plate: usize,
    group: &[usize],
    dist_table: &[Vec<usize>],
) -> Vec<usize> {
    let mut scored_points: Vec<(usize, usize)> = Vec::new();
    let r_target = target_plate / n;
    let c_target = target_plate % n;

    for x in 0..n * n {
        if !floor[x] { continue; }
        if !board.is_empty(x) && !group.contains(&x) && x != target_plate { continue; }
        if dist_table[x][target_plate] == usize::MAX { continue; }

        let mut cost = dist_table[x][target_plate];
        let mut possible = true;
        for &p in group {
            let d = dist_table[p][x];
            if d == usize::MAX {
                possible = false;
                break;
            }
            cost += d;
        }

        if possible {
            let r_x = x / n;
            let c_x = x % n;
            let on_axis = (r_x == r_target) || (c_x == c_target);
            let eval_cost = if on_axis { cost.saturating_sub(2) } else { cost };
            scored_points.push((eval_cost, x));
        }
    }

    scored_points.sort_by_key(|&(cost, _)| cost);

    let mut candidates = Vec::new();
    for &(_, x) in scored_points.iter().take(2) {
        if !candidates.contains(&x) {
            candidates.push(x);
        }
    }

    if !candidates.contains(&target_plate) && dist_table[target_plate][target_plate] == 0 {
        candidates.push(target_plate);
    }

    if candidates.is_empty() {
        candidates.push(target_plate);
    }

    candidates
}

fn apply_macro_action_with_point(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    _floor: &[bool],
    plates: &[usize],
    dist_table: &[Vec<usize>],
    board: &mut BitBoard,
    ops: &mut Vec<Op>,
    color: usize,
    group_max_size: usize,
    best_x: usize,
    ctx: &mut BfsContext,
) -> bool {
    let target_plate = plates[color];
    let mut positions = [0usize; 400];
    let mut pos_count = 0;
    for p in 0..n * n {
        if board.last(p) == Some(color) {
            positions[pos_count] = p;
            pos_count += 1;
        }
    }

    for i in 0..pos_count {
        let p = positions[i];
        if try_direct_nest_launch_fast(n, dir_step, max_ray, board, plates, ops, p, color) {
            return true;
        }
    }

    for i in 0..pos_count {
        let p = positions[i];
        if try_hop_and_launch_fast(n, dir_step, max_ray, board, plates, ops, p, color) {
            return true;
        }
    }

    if pos_count == 0 {
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
                    return true;
                }
            }
        }
        return false;
    }

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

    for &p in group {
        if p == best_x {
            continue;
        }
        let success = move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, p, best_x, ctx);
        if !success {
            move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, p, target_plate, ctx);
        }
    }

    if best_x != target_plate && !board.is_empty(best_x) {
        let success = move_tower_merge_fast(n, dir_step, max_ray, board, plates, ops, best_x, target_plate, ctx);
        if !success {
            let mut fallback_count = 0;
            while !board.is_empty(best_x) && fallback_count < 10 {
                let s = move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, best_x, target_plate, ctx);
                if !s { break; }
                fallback_count += 1;
            }
        }
    }

    eject_nest_blockers_fast(n, dir_step, max_ray, board, plates, ops, target_plate, color);

    true
}

fn apply_fallback_action(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    ctx: &mut BfsContext,
) -> bool {
    for p in 0..n * n {
        if let Some(color) = board.last(p) {
            let target_plate = plates[color];
            if move_single_trampoline_fast(n, dir_step, max_ray, board, plates, ops, p, target_plate, ctx) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone)]
struct BeamNode {
    board: BitBoard,
    ops: Vec<Op>,
    eval: usize,
    remaining_slimes: usize,
    hash: u64,
    color_history: Vec<usize>,
}

#[inline(always)]
fn compute_board_hash(board: &BitBoard) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for p in 0..400 {
        if board.lens[p] > 0 {
            h ^= (p as u64)
                .wrapping_mul(0x517cc1b727220a95)
                .wrapping_add((board.colors[p] as u64) << 16)
                .wrapping_add(board.lens[p] as u64);
            h = h.rotate_left(5) ^ 0x9e3779b97f4a7c15;
        }
    }
    h
}

fn evaluate_node(
    board: &BitBoard,
    cost: usize,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    k: usize,
) -> (usize, usize) {
    let mut total_dist = 0;
    let mut slime_count = 0;
    let mut max_dist_per_color = [0usize; 16];
    let mut count_per_color = [0usize; 16];
    let mut isolated_slimes = 0;
    let mut tower_slimes = 0;

    for p in 0..400 {
        let l = board.len(p);
        if l > 0 {
            slime_count += l;
            if l == 1 {
                isolated_slimes += 1;
            } else if l >= 4 {
                tower_slimes += l;
            }
            let mut c_bits = board.colors[p];
            for _ in 0..l {
                let color = (c_bits & 0xF) as usize;
                let target = plates[color];
                let d = dist_table[p][target];
                let safe_d = if d == usize::MAX { 50 } else { d };
                total_dist += safe_d;
                count_per_color[color] += 1;
                if safe_d > max_dist_per_color[color] {
                    max_dist_per_color[color] = safe_d;
                }
                c_bits >>= 4;
            }
        }
    }

    if slime_count == 0 {
        return (cost, 0);
    }

    let mut color_max_sum = 0;
    for c in 0..k {
        if count_per_color[c] > 0 {
            color_max_sum += max_dist_per_color[c];
        }
    }

    let est_remaining_cost = total_dist * 2 
        + color_max_sum * 4 
        + slime_count * 6 
        + isolated_slimes * 3
        - tower_slimes * 2;

    let eval = cost + est_remaining_cost;
    (eval, slime_count)
}

fn run_beam_search(
    n: usize,
    k: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    floor: &[bool],
    init_board: &BitBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    beam_width: usize,
    start_time: Instant,
    phase1_limit: std::time::Duration,
    ctx: &mut BfsContext,
    best_overall_score: &mut usize,
    best_overall_ops: &mut Vec<Op>,
    best_color_order: &mut Vec<usize>,
    beam_iter_count: &mut usize,
) -> bool {
    let (init_eval, init_slimes) = evaluate_node(init_board, 0, plates, dist_table, k);
    let init_node = BeamNode {
        board: *init_board,
        ops: Vec::with_capacity(300),
        eval: init_eval,
        remaining_slimes: init_slimes,
        hash: compute_board_hash(init_board),
        color_history: Vec::new(),
    };

    let mut current_beam = vec![init_node];
    let max_depth = 60;

    for _depth in 0..max_depth {
        if start_time.elapsed() >= phase1_limit {
            return false;
        }

        let mut next_candidates = Vec::new();
        let mut all_completed = true;

        for node in &current_beam {
            *beam_iter_count += 1;
            if node.remaining_slimes == 0 {
                if node.ops.len() < *best_overall_score {
                    *best_overall_score = node.ops.len();
                    *best_overall_ops = node.ops.clone();

                    let mut order = node.color_history.clone();
                    for c in 0..k {
                        if !order.contains(&c) {
                            order.push(c);
                        }
                    }
                    *best_color_order = order;
                }
                continue;
            }

            all_completed = false;

            let mut available_colors = Vec::new();
            for c in 0..k {
                let mut max_d = 0;
                let mut exists = false;
                let target = plates[c];
                for p in 0..n * n {
                    if node.board.contains(p, c) {
                        exists = true;
                        let d = dist_table[p][target];
                        if d != usize::MAX && d > max_d {
                            max_d = d;
                        }
                    }
                }
                if exists {
                    available_colors.push((c, max_d));
                }
            }

            available_colors.sort_by_key(|&(_, max_d)| std::cmp::Reverse(max_d));

            let mut branch_generated = false;

            for &(color, _) in &available_colors {
                let target_plate = plates[color];
                let mut positions = [0usize; 400];
                let mut pos_count = 0;
                for p in 0..n * n {
                    if node.board.last(p) == Some(color) {
                        positions[pos_count] = p;
                        pos_count += 1;
                    }
                }

                let mut seed_p = if pos_count > 0 { positions[0] } else { target_plate };
                let mut best_dist = 0;
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

                for &group_size in &[5, 8] {
                    let actual_group_size = pos_count.min(group_size);
                    let group = &sorted_pos[..actual_group_size];

                    let candidate_points = find_candidate_aggregation_points(
                        n,
                        floor,
                        &node.board,
                        target_plate,
                        group,
                        dist_table,
                    );

                    for &best_x in &candidate_points {
                        let mut next_board = node.board;
                        let mut next_ops = node.ops.clone();

                        let progressed = apply_macro_action_with_point(
                            n,
                            dir_step,
                            max_ray,
                            floor,
                            plates,
                            dist_table,
                            &mut next_board,
                            &mut next_ops,
                            color,
                            group_size,
                            best_x,
                            ctx,
                        );

                        if progressed && next_ops.len() > node.ops.len() {
                            let (eval, rem_slimes) = evaluate_node(&next_board, next_ops.len(), plates, dist_table, k);
                            let hash = compute_board_hash(&next_board);

                            let mut next_history = node.color_history.clone();
                            if !next_history.contains(&color) {
                                next_history.push(color);
                            }

                            next_candidates.push(BeamNode {
                                board: next_board,
                                ops: next_ops,
                                eval,
                                remaining_slimes: rem_slimes,
                                hash,
                                color_history: next_history,
                            });
                            branch_generated = true;
                        }
                    }
                }
            }

            if !branch_generated {
                let mut next_board = node.board;
                let mut next_ops = node.ops.clone();
                if apply_fallback_action(n, dir_step, max_ray, &mut next_board, plates, &mut next_ops, ctx) {
                    let (eval, rem_slimes) = evaluate_node(&next_board, next_ops.len(), plates, dist_table, k);
                    let hash = compute_board_hash(&next_board);
                    next_candidates.push(BeamNode {
                        board: next_board,
                        ops: next_ops,
                        eval,
                        remaining_slimes: rem_slimes,
                        hash,
                        color_history: node.color_history.clone(),
                    });
                }
            }
        }

        if all_completed || next_candidates.is_empty() {
            break;
        }

        next_candidates.sort_by_key(|n| n.eval);

        let mut filtered_beam = Vec::with_capacity(beam_width);
        let mut seen_hashes = std::collections::HashSet::new();

        for cand in next_candidates {
            if seen_hashes.insert(cand.hash) {
                if cand.remaining_slimes == 0 {
                    if cand.ops.len() < *best_overall_score {
                        *best_overall_score = cand.ops.len();
                        *best_overall_ops = cand.ops.clone();
                        let mut order = cand.color_history.clone();
                        for c in 0..k {
                            if !order.contains(&c) {
                                order.push(c);
                            }
                        }
                        *best_color_order = order;
                    }
                }
                filtered_beam.push(cand);
                if filtered_beam.len() >= beam_width {
                    break;
                }
            }
        }

        current_beam = filtered_beam;
    }

    true
}

// -------------------------------------------------------------
// メイン処理（Stage 1 事前集約 ＋ 案B 高度化ハイブリッド）
// -------------------------------------------------------------
fn main() {
    let start_time = Instant::now();
    let total_time_limit = std::time::Duration::from_millis(1850);
    let phase1_limit = std::time::Duration::from_millis(600);

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

    // ---------------------------------------------------------
    // Phase 1: 多段階ビームサーチ（0.0s 〜 0.6s）
    // ---------------------------------------------------------
    let mut beam_best_score = usize::MAX;
    let mut beam_best_ops = Vec::new();
    let mut seed_color_order: Vec<usize> = (0..k).collect();
    let mut beam_iter_count = 0;
    let beam_widths = [10, 20, 35, 60, 90];

    for &w in &beam_widths {
        if start_time.elapsed() >= phase1_limit {
            break;
        }
        let finished = run_beam_search(
            n,
            k,
            &dir_step,
            &max_ray,
            &floor,
            &init_board,
            &plates,
            &dist_table,
            w,
            start_time,
            phase1_limit,
            &mut ctx,
            &mut beam_best_score,
            &mut beam_best_ops,
            &mut seed_color_order,
            &mut beam_iter_count,
        );
        if !finished {
            break;
        }
    }

    // ---------------------------------------------------------
    // Phase 2: 焼きなまし法（0.6s 〜 1.85s）
    // Stage 1 (事前集約モード: 0: なし, 1: 局所ペアマージ, 2: 中間ハブ集約)
    // ---------------------------------------------------------
    let mut rng = XorShift::new(20260925);
    let mut current_order = seed_color_order.clone();
    let mut current_group_sizes = [5usize; 16];
    let mut current_strategies = [0u8; 16];
    let mut current_stage1_modes = [0u8; 16]; // 初期状態はモード0

    let (mut current_score, mut sa_best_ops) = solve_with_stages(
        n,
        k,
        &dir_step,
        &max_ray,
        &floor,
        &init_board,
        &plates,
        &dist_table,
        &current_order,
        &current_group_sizes,
        &current_strategies,
        &current_stage1_modes,
        &mut ctx,
    );
    let mut sa_best_score = current_score;

    let t_start: f64 = 8.0;
    let t_end: f64 = 0.5;
    let mut sa_iter_count = 0;

    while start_time.elapsed() < total_time_limit {
        sa_iter_count += 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let total = total_time_limit.as_secs_f64();
        let elapsed_ratio = (elapsed / total).min(1.0);
        let temperature = t_start * (1.0 - elapsed_ratio) + t_end * elapsed_ratio;

        let mut next_order = current_order.clone();
        let mut next_group_sizes = current_group_sizes;
        let mut next_strategies = current_strategies;
        let mut next_stage1_modes = current_stage1_modes;

        let move_type = rng.gen_range(0, 100);
        if move_type < 20 {
            // 2色スワップ
            let i = rng.gen_range(0, k);
            let j = rng.gen_range(0, k);
            next_order.swap(i, j);
        } else if move_type < 40 {
            // 1色挿入
            let from = rng.gen_range(0, k);
            let to = rng.gen_range(0, k);
            let val = next_order.remove(from);
            next_order.insert(to, val);
        } else if move_type < 55 {
            // 区間反転
            let mut i = rng.gen_range(0, k);
            let mut j = rng.gen_range(0, k);
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            next_order[i..=j].reverse();
        } else if move_type < 70 {
            // グループサイズ個別変更 (3..=8)
            let c = rng.gen_range(0, k);
            next_group_sizes[c] = rng.gen_range(3, 9);
        } else if move_type < 85 {
            // 集約戦略個別変更 (0: コスト最小, 1: 軸上優先, 2: 巣そのもの)
            let c = rng.gen_range(0, k);
            next_strategies[c] = rng.gen_range(0, 3) as u8;
        } else {
            // Stage 1 事前集約モード変更 (0: なし, 1: 局所ペアマージ, 2: 中間ハブ集約)
            let c = rng.gen_range(0, k);
            next_stage1_modes[c] = rng.gen_range(0, 3) as u8;
        }

        let (cand_score, cand_ops) = solve_with_stages(
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
            &next_strategies,
            &next_stage1_modes,
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
            current_strategies = next_strategies;
            current_stage1_modes = next_stage1_modes;
            current_score = cand_score;

            if cand_score < sa_best_score {
                sa_best_score = cand_score;
                sa_best_ops = cand_ops;
            }
        }
    }

    // ---------------------------------------------------------
    // 最終決定：ビームサーチ解とSA解の優れた方を採用
    // ---------------------------------------------------------
    let (final_score, final_ops, winner) = if beam_best_score < sa_best_score {
        (beam_best_score, beam_best_ops, "BeamSearch")
    } else {
        (sa_best_score, sa_best_ops, "SimulatedAnnealing")
    };

    eprintln!(
        "STAT: beam_iters={} sa_iters={} beam_score={} sa_score={} winner={} final_score={}",
        beam_iter_count, sa_iter_count, beam_best_score, sa_best_score, winner, final_score
    );

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &final_ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
