// C_01: B_09 + differential validation + complete bridges + compact exact search.
// -------------------------------------------------------------
// AHC 072 - B_03: 2-Tier Adaptive Solver (A44 + B01)
// 1. 通常盤面 (slimes <= 120 または slimes <= 145 && walls < 100):
//    A44 (Tier最適排他事前集約 + カタパルト中継砲台 + 1.85s 焼きなまし SA + Pruning)
// 2. 超密集盤面 (それ以外):
//    B01 (純粋高速 Chokudai Search + 3手先静穏先読み + Pruning + LNS)
// -------------------------------------------------------------

use std::io::{self, Read};

pub const TH_SMALL: usize = 75;
pub const TH_MID: usize = 115;

#[derive(Clone, Copy, Debug)]
pub struct TunableParams {
    pub th_small: usize,
    pub th_mid: usize,
    pub t_start: f64,
    pub t_end: f64,
    pub w_strat_tree: usize,
    pub w_strat_sweep: usize,
    pub w_strat_catapult: usize,
    pub w_strat_centroid: usize,
    pub p_strategy: usize,
}

impl Default for TunableParams {
    fn default() -> Self {
        Self {
            th_small: TH_SMALL,
            th_mid: TH_MID,
            t_start: 10.0,
            t_end: 0.3,
            w_strat_tree: 3,
            w_strat_sweep: 5,
            w_strat_catapult: 3,
            w_strat_centroid: 2,
            p_strategy: 5,
        }
    }
}

pub fn parse_global_args() -> TunableParams {
    let mut params = TunableParams::default();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--th_small" => { i += 1; if i < args.len() { params.th_small = args[i].parse().unwrap_or(TH_SMALL); } }
            "--th_mid" => { i += 1; if i < args.len() { params.th_mid = args[i].parse().unwrap_or(TH_MID); } }
            "--t_start" => { i += 1; if i < args.len() { params.t_start = args[i].parse().unwrap_or(10.0); } }
            "--t_end" => { i += 1; if i < args.len() { params.t_end = args[i].parse().unwrap_or(0.3); } }
            "--w_strat_tree" => { i += 1; if i < args.len() { params.w_strat_tree = args[i].parse().unwrap_or(3); } }
            "--w_strat_sweep" => { i += 1; if i < args.len() { params.w_strat_sweep = args[i].parse().unwrap_or(5); } }
            "--w_strat_catapult" => { i += 1; if i < args.len() { params.w_strat_catapult = args[i].parse().unwrap_or(3); } }
            "--w_strat_centroid" => { i += 1; if i < args.len() { params.w_strat_centroid = args[i].parse().unwrap_or(2); } }
            "--p_strategy" => { i += 1; if i < args.len() { params.p_strategy = args[i].parse().unwrap_or(5); } }
            _ => {}
        }
        i += 1;
    }
    params
}

mod a44 {
use std::io::{self, Write};
use std::time::Instant;

const DIR: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const DIR_CHAR: [char; 4] = ['U', 'D', 'L', 'R'];

#[derive(Clone, Copy, Debug)]
pub struct Op {
    pub r: usize,
    pub c: usize,
    pub k: usize,
    pub d: char,
    pub l: usize,
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
pub struct BitBoard {
    colors: [u32; 400],
    lens: [u8; 400],
}

impl BitBoard {
    pub fn new(n: usize, init_stacks: &[Vec<usize>]) -> Self {
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn is_pure_single_color(&self, p: usize) -> Option<usize> {
        let l = self.lens[p] as usize;
        if l == 0 { return None; }
        let c0 = (self.colors[p] & 0xF) as usize;
        let mut temp = self.colors[p];
        for _ in 0..l {
            if (temp & 0xF) as usize != c0 {
                return None;
            }
            temp >>= 4;
        }
        Some(c0)
    }

    #[inline(always)]
    fn can_accept_color_on_top(&self, q: usize, color: usize) -> bool {
        let l = self.lens[q] as usize;
        if l == 0 { return true; }
        let mut temp = self.colors[q];
        for i in 0..l {
            let c = (temp & 0xF) as usize;
            if c == color && i < l - 1 {
                return false;
            }
            temp >>= 4;
        }
        true
    }


    #[inline(always)]
    fn transfer_and_reverse(&mut self, p: usize, next_p: usize, k: usize) {
        if p >= 400 || next_p >= 400 {
            panic!("INVALID transfer_and_reverse: p={}, next_p={}, k={}", p, next_p, k);
        }
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
}

pub struct BfsContext {
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
    queue_tower: [usize; 4096],
    path_tower: [usize; 400],
    path_tower_backup: [usize; 400],
    allow_01_detour: bool,
    init_total_slimes: usize,
    density: f64,
}

impl BfsContext {
    pub fn new() -> Self {
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
            queue_tower: [0; 4096],
            path_tower: [0; 400],
            path_tower_backup: [0; 400],
            allow_01_detour: true,
            init_total_slimes: 0,
            density: 0.0,
        }
    }
}

pub fn compute_max_ray(n: usize, floor: &[bool]) -> [[usize; 4]; 400] {
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

pub fn compute_all_pairs_dist(n: usize, floor: &[bool]) -> Vec<Vec<usize>> {
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

// -------------------------------------------------------------
// Step 15 の強力なタワー合体歩行 BFS（find_path_merge_fast）
// 途中マスにスライムがいても、合体して <= 8 なら通過可能！
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
    // 1. 純粋 BFS で最短歩数を探索
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

    let mut pure_step_count = 0;
    let mut curr_idx = target_idx;
    let mut pure_absorbed_same_color = false;
    while curr_idx != start_idx && pure_step_count < 400 {
        let (prev_idx, d) = ctx.prev_tower[curr_idx];
        ctx.path_tower_backup[pure_step_count] = d;
        let u = curr_idx / 9;
        if u != target && board.len(u) > 0 && board.last(u) == target_color {
            pure_absorbed_same_color = true;
        }
        pure_step_count += 1;
        curr_idx = prev_idx;
    }

    // もし迂回が無効、または最短路の時点ですでに同色スライムを吸収できているなら、純粋最短路をそのまま採用！
    if !ctx.allow_01_detour || pure_absorbed_same_color {
        for i in 0..pure_step_count {
            ctx.path_tower[i] = ctx.path_tower_backup[pure_step_count - 1 - i];
        }
        return Some(pure_step_count);
    }

    // 2. 0-1 BFS で同色スライム吸収を優先する迂回ルートを探索
    ctx.token_tower += 1;
    let token_01 = ctx.token_tower;

    ctx.visited_tower[start_idx] = token_01;
    ctx.dist_tower[start_idx] = 0;

    let mut head = 2048;
    let mut tail = 2048;
    ctx.queue_tower[tail] = start_idx;
    tail = (tail + 1) & 4095;

    let mut best_target_idx_01 = None;

    while head != tail {
        let curr_idx = ctx.queue_tower[head];
        head = (head + 1) & 4095;

        let u = curr_idx / 9;
        let h = curr_idx % 9;

        if u == target {
            best_target_idx_01 = Some(curr_idx);
            break;
        }

        for d in 0..4 {
            if max_ray[u][d] < 1 {
                continue;
            }
            let v = (u as isize + dir_step[d]) as usize;

            let (next_h, weight) = if v == target {
                if board.len(v) + h > 8 {
                    continue;
                }
                (h, 1)
            } else {
                let v_len = board.len(v);
                let nh = h + v_len;
                if nh > 8 {
                    continue;
                }
                let w = if v_len > 0 && board.last(v) == target_color { 0 } else { 1 };
                (nh, w)
            };

            let next_idx = v * 9 + next_h;
            let new_dist = ctx.dist_tower[curr_idx] + weight;

            if ctx.visited_tower[next_idx] != token_01 || new_dist < ctx.dist_tower[next_idx] {
                ctx.visited_tower[next_idx] = token_01;
                ctx.dist_tower[next_idx] = new_dist;
                ctx.prev_tower[next_idx] = (curr_idx, d);

                if weight == 0 {
                    head = (head + 4095) & 4095;
                    ctx.queue_tower[head] = next_idx;
                } else {
                    ctx.queue_tower[tail] = next_idx;
                    tail = (tail + 1) & 4095;
                }
            }
        }
    }

    if let Some(target_idx_01) = best_target_idx_01 {
        let mut step_count_01 = 0;
        let mut curr_idx_01 = target_idx_01;
        while curr_idx_01 != start_idx && step_count_01 < 400 {
            let (prev_idx, d) = ctx.prev_tower[curr_idx_01];
            ctx.path_tower[step_count_01] = d;
            step_count_01 += 1;
            curr_idx_01 = prev_idx;
        }

        // 迂回歩数が「最短路 + 2手以内」なら 0-1 BFS を採用！
        if step_count_01 <= pure_step_count + 2 {
            ctx.path_tower[..step_count_01].reverse();
            return Some(step_count_01);
        }
    }

    // 迂回が大きすぎる、または見つからない場合は純粋最短路を採用
    for i in 0..pure_step_count {
        ctx.path_tower[i] = ctx.path_tower_backup[pure_step_count - 1 - i];
    }
    Some(pure_step_count)
}

// -------------------------------------------------------------
// 巣穴直前・途中の異色同乗スライム前方射出（Staging Pre-Nest Ejection）
// 目的の巣穴に向かうタワーのトップに異色スライムが乗っている場合、
// 巣穴に一緒に運ぶ前に、その異色スライムの目的地方向へ先にジャンプ射出・切り離す！
// -------------------------------------------------------------
fn try_eject_co_riders_before_nest(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    curr: usize,
    target_plate: usize,
    target_color: usize,
    dist_table: &[Vec<usize>],
) -> bool {
    let h = board.len(curr);
    if h < 2 || curr == target_plate {
        return false;
    }

    let (top_count, top_color) = board.count_top_consecutive(curr);
    if top_color == target_color {
        return false; // トップが目的色なら射出不要
    }

    let k = h - top_count;
    if k == 0 {
        return false; // 土台がない
    }

    // 土台の中に target_color が含まれているか？
    let mut base_has_target = false;
    let mut c = board.colors[curr];
    for _ in 0..k {
        if ((c & 0xF) as usize) == target_color {
            base_has_target = true;
            break;
        }
        c >>= 4;
    }
    if !base_has_target {
        return false;
    }

    let max_l = (k + 1).min(8);
    let other_target = plates[top_color];
    let curr_dist = dist_table[curr][other_target];

    let mut best_action = None;
    let mut best_score = isize::MIN;

    for d in 0..4 {
        let limit = max_l.min(max_ray[curr][d]);
        let step = dir_step[d];
        let mut v = curr as isize;

        for l in 1..=limit {
            v += step;
            let next_p = v as usize;

            if next_p == target_plate {
                continue; // 目的の巣穴に異色を投げ込むのは禁止
            }
            if board.len(next_p) + top_count > 8 {
                continue; // 高さ超過
            }

            let next_dist = dist_table[next_p][other_target];
            if next_dist == usize::MAX {
                continue;
            }

            let landing_len = board.len(next_p);

            let mut score: isize = 0;
            if next_p == other_target {
                score = 20000 + (l as isize) * 10;
            } else if landing_len > 0 && board.last(next_p) == Some(top_color) {
                score = 10000 + (l as isize) * 10 - (next_dist as isize) * 5;
            } else if next_dist < curr_dist {
                let dist_saved = (curr_dist - next_dist) as isize;
                score = 1000 + dist_saved * 20 + (l as isize) * 5;
            } else if next_dist == curr_dist && l >= 1 {
                let d_to_target = dist_table[next_p][target_plate];
                let curr_d_to_target = dist_table[curr][target_plate];
                if d_to_target > curr_d_to_target {
                    score = 200 + (l as isize) * 5;
                }
            }

            if score > 0 && score > best_score {
                best_score = score;
                best_action = Some((d, l, next_p));
            }
        }
    }

    if let Some((d, l, next_p)) = best_action {
        ops.push(Op {
            r: curr / n,
            c: curr % n,
            k,
            d: DIR_CHAR[d],
            l,
        });

        board.transfer_and_reverse(curr, next_p, k);

        for &chk in &[curr, next_p] {
            if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                board.clean_nest(chk, pc);
            }
        }

        return true;
    }

    false
}

// -------------------------------------------------------------
// Step 19 統合版：最短移動プラン探索エンジン（move_smart_transfer）
// 1. h == 1: 直接単体ジャンプ輸送（高速＆飛び越え）
// 2. h >= 2:
//    - (A) 直線大ジャンプ（途中のスライムタワー上空を飛び越えて一撃ダンク/射出）
//    - (B) タワー合体歩行（途中スライム巻き込み合体OKの柔軟なBFS）
//    - (C) 単体・分割輸送（フォールバック）
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
    dist_table: &[Vec<usize>],
    ctx: &mut BfsContext,
) -> bool {
    if start == target || board.is_empty(start) {
        return true;
    }

    // 目的が巣穴の場合、移動開始前にトップの異色スライムを先行射出！
    if let Some(target_color) = plates.iter().position(|&p| p == target) {
        try_eject_co_riders_before_nest(n, dir_step, max_ray, board, plates, ops, start, target, target_color, dist_table);
        if start == target || board.is_empty(start) {
            return true;
        }
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
        // 壁さえなければ、途中にどれだけタワーがあっても上空を飛び越えて直撃可能！
        if max_ray[start][d] >= dist_l {
            // 隣のマス（dist_l == 1）：タワー丸ごと一撃移動（1手！）
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
            // 【完全統合ソフトモデル：Tier最適化×密度適応シグモイド＆費用便益比ハイブリッド】
            let rho = ctx.density;
            let slimes = ctx.init_total_slimes;

            let max_launch_dist = if slimes > 115 {
                // Dense帯：浸透閾値超過のため、物理閉塞を防ぐため厳格に3マス固定
                3
            } else if slimes > 75 {
                // Medium帯：混雑度 rho に応じて 4〜8マスを滑らかに許可
                let sig_rho = 1.0 / (1.0 + ((rho - 0.45) / 0.03).exp());
                4 + (4.0 * sig_rho).round() as usize
            } else {
                // Small帯：基本は3マス、適正密度 (rho ≈ 0.285) のスイートスポットのみ 4〜5マスを許可
                let diff = rho - 0.285;
                let gaussian = (-diff * diff / (2.0 * 0.045 * 0.045)).exp();
                3 + (2.0 * gaussian).round() as usize
            };

            let k_est = dist_l.saturating_sub(1);
            let cost_k = (k_est as f64) * (0.3 + 0.8 * rho) + ((k_est.saturating_sub(2)) as f64).powi(2) * (1.0 + 2.0 * rho);
            let benefit = ((dist_l - 1) as f64) - cost_k;

            let is_target_nest = plates.contains(&target);
            let is_start_nest = plates.contains(&start) && ctx.init_total_slimes > 75;

            if dist_l > 1 && dist_l <= max_launch_dist && benefit >= -0.5 && h >= dist_l && (is_target_nest || is_start_nest) {
                let k = dist_l - 1;
                let flying = h - k;
                if board.len(target) + flying <= 8 {
                    let mut can_launch = false;
                    if is_target_nest {
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
                            can_launch = true;
                        }
                    } else if is_start_nest {
                        let start_color = plates.iter().position(|&pos| pos == start).unwrap();
                        let mut base_has_start = false;
                        let mut c = board.colors[start];
                        for _ in 0..k {
                            if ((c & 0xF) as usize) == start_color {
                                base_has_start = true;
                                break;
                            }
                            c >>= 4;
                        }
                        if base_has_start {
                            can_launch = true;
                        }
                    }

                    if can_launch {
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

    // 2. (B) タワー合体歩行（Step 15 の強力な合体許容 BFS！）
    // 途中マスにスライムがいても巻き込み合体して通過できるため、超密集ケースでも詰まらない！
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

            // 巣穴へ向かう道中で異色スライムが乗った場合の前方射出！
            if let Some(target_color) = plates.iter().position(|&p| p == target) {
                if curr != target {
                    try_eject_co_riders_before_nest(n, dir_step, max_ray, board, plates, ops, curr, target, target_color, dist_table);
                    if board.is_empty(curr) {
                        break;
                    }
                }
            }

            // 方針 3: 同乗色コンソリデーション（Co-riding Color Consolidation）
            let h_curr = board.len(curr);
            if h_curr > 0 && h_curr < 8 && curr != target {
                // 未来のステップでタワーが高さ8を超えないかを事前検証するヘルパー
                let can_safely_consolidate = |board: &BitBoard, h_add: usize, curr_pos: usize| -> bool {
                    let mut sim_h = board.len(curr_pos) + h_add;
                    if sim_h > 8 {
                        return false;
                    }
                    let mut sim_pos = curr_pos;
                    for f_idx in (idx + 1)..step_count {
                        let d = ctx.path_tower[f_idx];
                        let next_pos = (sim_pos as isize + dir_step[d]) as usize;
                        if next_pos >= 400 {
                            return false;
                        }
                        let dest_h = board.len(next_pos);
                        if dest_h > 0 {
                            sim_h += dest_h;
                            if sim_h > 8 {
                                return false;
                            }
                        }
                        sim_pos = next_pos;
                    }
                    true
                };

                for d_sweep in 0..4 {
                    if board.len(curr) >= 8 {
                        break;
                    }

                    let d_in = match d_sweep {
                        0 => 1,
                        1 => 0,
                        2 => 3,
                        3 => 2,
                        _ => 0,
                    };

                    let r = curr / n;
                    let c = curr % n;
                    let (dr, dc) = [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)][d_sweep];

                    // 1. 直線 1 マス先（隣接マス）のチェック
                    let r1 = r as isize + dr;
                    let c1 = c as isize + dc;
                    if r1 >= 0 && r1 < n as isize && c1 >= 0 && c1 < n as isize && max_ray[curr][d_sweep] >= 1 {
                        let adj1 = r1 as usize * n + c1 as usize;
                        if !plates.contains(&adj1) && !board.is_empty(adj1) {
                            let h_adj = board.len(adj1);
                            if can_safely_consolidate(board, h_adj, curr) {
                                if let Some(col) = board.last(adj1) {
                                    if board.contains(curr, col) && board.can_accept_color_on_top(curr, col) {
                                        ops.push(Op {
                                            r: adj1 / n,
                                            c: adj1 % n,
                                            k: 0,
                                            d: DIR_CHAR[d_in],
                                            l: 1,
                                        });
                                        board.transfer_and_reverse(adj1, curr, 0);
                                        continue;
                                    }
                                }
                            }
                        }
                    }

                    // 2. 直線 2 マス先のチェック（中間マスが空き床の場合）
                    let r2 = r as isize + dr * 2;
                    let c2 = c as isize + dc * 2;
                    if r2 >= 0 && r2 < n as isize && c2 >= 0 && c2 < n as isize && max_ray[curr][d_sweep] >= 2 {
                        let adj1 = (r as isize + dr) as usize * n + (c as isize + dc) as usize;
                        let adj2 = r2 as usize * n + c2 as usize;
                        if board.is_empty(adj1) && !plates.contains(&adj1) && !plates.contains(&adj2) && !board.is_empty(adj2) {
                            let h_adj = board.len(adj2);
                            if can_safely_consolidate(board, h_adj, curr) {
                                if let Some(col) = board.last(adj2) {
                                    if board.contains(curr, col) && board.can_accept_color_on_top(curr, col) {
                                        // adj2 -> adj1 -> curr
                                        ops.push(Op {
                                            r: adj2 / n,
                                            c: adj2 % n,
                                            k: 0,
                                            d: DIR_CHAR[d_in],
                                            l: 1,
                                        });
                                        board.transfer_and_reverse(adj2, adj1, 0);

                                        ops.push(Op {
                                            r: adj1 / n,
                                            c: adj1 % n,
                                            k: 0,
                                            d: DIR_CHAR[d_in],
                                            l: 1,
                                        });
                                        board.transfer_and_reverse(adj1, curr, 0);
                                    }
                                }
                            }
                        }
                    }
                }
            }

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

// 改善2: マルチスライム一括射出（Bulk Direct Launch）
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
    if k > 0 {
        // 土台 k 匹の中に目的の色（color）と同じ色が含まれているか？
        let mut base_has_target = false;
        let mut c = board.colors[p];
        for _ in 0..k {
            if ((c & 0xF) as usize) == color {
                base_has_target = true;
                break;
            }
            c >>= 4;
        }
        if base_has_target {
            // 同色を土台に残すジャンプは後から回収コストがかかるので禁止！
            return false;
        }
    }
    let flying_count = h - k;
    if board.len(target) + flying_count > 8 {
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

// 改善2: マルチスライム2段跳躍一括帰巣（Bulk Hop & Shot）
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
            if m == target || plates.contains(&m) {
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
    dist_table: &[Vec<usize>],
) {
    let mut eject_count = 0;
    while !board.is_empty(nest) && eject_count < 10 {
        if board.last(nest) == Some(nest_color) {
            board.clean_nest(nest, nest_color);
            if board.is_empty(nest) {
                break;
            }
        }

        let (count, flying_color) = board.count_top_consecutive(nest);
        let h = board.len(nest);
        let k = h - count;
        let max_l = k + 1;

        let target_for_flying = plates[flying_color];
        let mut best_action = None;
        let mut best_val = isize::MIN;

        for d in 0..4 {
            let limit = max_l.min(max_ray[nest][d]);
            let step = dir_step[d];
            let mut v = nest as isize;

            for l in 1..=limit {
                v += step;
                let next_p = v as usize;
                if board.len(next_p) + count <= 8 {
                    let d_actual = dist_table[next_p][target_for_flying];
                    if d_actual == usize::MAX {
                        continue;
                    }
                    // Prefer closer to destination nest in actual path distance, with jump distance bonus
                    let val = -(d_actual as isize) * 10 + (l as isize) * 3;
                    if val > best_val {
                        best_val = val;
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

        board.clean_nest(nest, nest_color);
        if let Some(pc) = plates.iter().position(|&pos| pos == next_p) {
            board.clean_nest(next_p, pc);
        }

        eject_count += 1;
    }
}

// -------------------------------------------------------------
// Step 38: 道中異色スライム大ジャンプ射出・デポ（Cross-Color Leap Ejection）
// 異色混合タワーの上部スライムを、土台を弾力にして目的地/合体相手/デポ地へ大ジャンプ！
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

            let next_dist_to_nest = dist_table[next_p][target_plate];
            let landing_len = board.len(next_p);

            // ケース A: 巣に直撃ダンク！
            if next_p == target_plate {
                let gain = 1000 + (l as isize) * 10;
                if gain > best_gain {
                    best_gain = gain;
                    best_action = Some((d, l, next_p));
                }
                continue;
            }

            // ケース B: 同色のスライムがいるマスに着地（合体！）
            if landing_len > 0 && board.last(next_p) == Some(top_color) {
                let gain = 100 + (l as isize) * 10;
                if gain > best_gain {
                    best_gain = gain;
                    best_action = Some((d, l, next_p));
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
                    let mut b = 35;
                    if on_axis_dist <= 4 {
                        b += 20;
                    }
                    b
                } else if r_next == r_tgt || c_next == c_tgt {
                    15
                } else {
                    0
                };

                let gain = dist_saved * 12 + (l as isize) * 4 + on_axis_bonus;
                if gain > 10 && gain > best_gain {
                    best_gain = gain;
                    best_action = Some((d, l, next_p));
                }
            }
        }
    }

    if let Some((d, l, next_p)) = best_action {
        ops.push(Op {
            r: p / n,
            c: p % n,
            k,
            d: DIR_CHAR[d],
            l,
        });

        board.transfer_and_reverse(p, next_p, k);

        for &chk in &[p, next_p] {
            if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                board.clean_nest(chk, pc);
            }
        }

        return true;
    }

    false
}

// -------------------------------------------------------------
// Stage 1: 本格的事前クラスタリング（Aggressive Full-Tower Pre-Clustering）
// move_smart_transfer を使って、タワーを崩さずに合体させる！
// -------------------------------------------------------------
fn pre_cluster_full_towers(
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
    let target_plate = plates[color];

    let mut total_slimes = 0;
    for p in 0..n * n {
        total_slimes += board.len(p);
    }

    // 【対策案C】Medium帯 (76〜115) 限定で近傍3マス以内に制限
    let max_allowed_dist = if ctx.init_total_slimes > 75 && ctx.init_total_slimes <= 115 {
        if total_slimes >= 100 { 2 } else { 3 }
    } else {
        if total_slimes >= 100 { 2 } else if total_slimes >= 60 { 4 } else { 7 }
    };

    let mut loop_cnt = 0;
    while loop_cnt < 8 {
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

        // Find the best pair (p, q) to merge p into q.
        // We want to merge slimes hierarchically:
        // p merges into q if q is closer to the target (or nearby) so that p saves travel distance.
        let mut best_pair = None;
        let mut max_gain = isize::MIN;

        for i in 0..pos_count {
            let p = positions[i];
            let h_p = board.len(p);
            let d_p = dist_table[p][target_plate];
            if d_p == usize::MAX { continue; }

            for j in 0..pos_count {
                if i == j { continue; }
                let q = positions[j];
                let h_q = board.len(q);
                if h_p + h_q > 8 { continue; }

                let d_q = dist_table[q][target_plate];
                if d_q == usize::MAX { continue; }

                let d_pq = dist_table[p][q];
                if d_pq == usize::MAX || d_pq > max_allowed_dist { continue; }

                // Net gain: distance saved by moving p to q rather than moving p to target independently
                // Gain = d_p - d_pq (higher is better)
                // Bonus if q is closer to target than p: d_p >= d_q
                let mut gain = (d_p as isize) - (d_pq as isize);
                if d_q <= d_p {
                    gain += 5; // Preference for moving towards target
                } else {
                    gain -= 10; // Penalty for moving away from target
                }
                // Tie-breaker: prefer higher destination tower q
                gain += h_q as isize;

                if gain > max_gain {
                    max_gain = gain;
                    best_pair = Some((p, q));
                }
            }
        }

        let Some((p, q)) = best_pair else {
            break;
        };

        // If even the best pair has non-positive gain and we are moving away from target, stop
        if max_gain < 0 {
            break;
        }

        let success = move_smart_transfer(n, dir_step, max_ray, board, plates, ops, p, q, dist_table, ctx);
        if !success {
            break;
        }
    }
}

fn pre_cluster_chunked_mega_towers(
    n: usize,
    dir_step: &[isize; 4],
    max_ray: &[[usize; 4]; 400],
    board: &mut BitBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    dist_table: &[Vec<usize>],
    ctx: &mut BfsContext,
    init_total_slimes: usize,
) {
    let mut total_slimes = 0;
    for p in 0..n * n {
        total_slimes += board.len(p);
    }
    let max_dist = if total_slimes <= 40 { 5 } else if total_slimes <= 75 { 3 } else { 2 };

    let mut loop_cnt = 0;
    while loop_cnt < 16 {
        loop_cnt += 1;
        let mut best_pair = None;
        let mut max_gain = isize::MIN;

        for p in 0..n * n {
            let h_p = board.len(p);
            if h_p == 0 || h_p >= 7 { continue; }

            for q in 0..n * n {
                if p == q { continue; }
                let h_q = board.len(q);
                if h_q == 0 || h_p + h_q > 7 { continue; }

                let d = dist_table[p][q];
                if d > max_dist || d == usize::MAX { continue; }

                // Small / Medium 帯（init_total_slimes <= 115 かつ total_slimes <= 75）限定で巣穴指向性ペナルティ・ボーナスを適用
                // Dense 帯（init_total_slimes > 115）はベースライン（nest_score = 0）を完全維持
                let nest_score = if init_total_slimes <= 115 && total_slimes <= 75 {
                    let mut nest_dist_delta = 0isize;
                    let mut c_bits = board.colors[p];
                    for _ in 0..h_p {
                        let col = (c_bits & 0xF) as usize;
                        c_bits >>= 4;
                        let target_nest = plates[col];
                        let d_p_nest = dist_table[p][target_nest];
                        let d_q_nest = dist_table[q][target_nest];
                        if d_p_nest != usize::MAX && d_q_nest != usize::MAX {
                            let diff = (d_p_nest as isize) - (d_q_nest as isize);
                            nest_dist_delta += diff;
                        }
                    }
                    if nest_dist_delta >= 0 {
                        nest_dist_delta * 15
                    } else {
                        nest_dist_delta * 30
                    }
                } else {
                    0
                };

                let combined_h = h_p + h_q;
                let gain = (combined_h as isize) * 8 - (d as isize) * 10 + nest_score;
                if gain > max_gain {
                    max_gain = gain;
                    best_pair = Some((p, q));
                }
            }
        }

        if let Some((p, q)) = best_pair {
            if max_gain <= 0 {
                break;
            }
            if !move_smart_transfer(n, dir_step, max_ray, board, plates, ops, p, q, dist_table, ctx) {
                break;
            }
        } else {
            break;
        }
    }
}

// -------------------------------------------------------------
// フェーズ 2 用: 完全統合シミュレーション関数（Stage 1 + Stage 2）
// -------------------------------------------------------------

fn pre_cluster_multi_color_for_color(
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
    let mut total_slimes = 0;
    for p in 0..n * n {
        total_slimes += board.len(p);
    }
    let max_allowed_dist = if total_slimes >= 100 { 2 } else if total_slimes >= 60 { 3 } else { 5 };

    let mut my_towers = [0usize; 32];
    let mut my_count = 0;
    for p in 0..n * n {
        if board.last(p) == Some(color) {
            let h = board.len(p);
            if h > 0 && h < 8 {
                my_towers[my_count] = p;
                my_count += 1;
                if my_count == 32 { break; }
            }
        }
    }
    if my_count == 0 {
        return;
    }

    let mut other_towers = [(0usize, 0usize); 64];
    let mut other_count = 0;
    for p in 0..n * n {
        let h = board.len(p);
        if h > 0 && h < 8 {
            other_towers[other_count] = (p, h);
            other_count += 1;
            if other_count == 64 { break; }
        }
    }
    if other_count < 2 {
        return;
    }

    let mut loop_cnt = 0;
    while loop_cnt < 16 {
        loop_cnt += 1;
        let mut best_pair = None;
        let mut max_gain = isize::MIN;

        for i in 0..my_count {
            let p = my_towers[i];
            let h_p = board.len(p);
            if h_p == 0 || h_p >= 8 { continue; }
            let r_p = (p / n) as isize;
            let c_p = (p % n) as isize;

            for j in 0..other_count {
                let (q, _) = other_towers[j];
                if p == q { continue; }
                let h_q = board.len(q);
                if h_q == 0 || h_p + h_q > 8 { continue; }

                let r_q = (q / n) as isize;
                let c_q = (q % n) as isize;
                let manhattan = (r_p - r_q).abs() + (c_p - c_q).abs();
                if manhattan > max_allowed_dist as isize { continue; }

                let d_pq = dist_table[p][q];
                if d_pq == usize::MAX || d_pq > max_allowed_dist { continue; }

                let combined_h = h_p + h_q;
                let mut gain = (combined_h as isize) * 3 - (d_pq as isize * 2);
                if combined_h >= 6 {
                    gain += 15;
                }
                if gain > max_gain {
                    max_gain = gain;
                    best_pair = Some((p, q));
                }
            }
        }

        let Some((p, q)) = best_pair else {
            break;
        };
        if max_gain <= 0 {
            break;
        }

        let success = move_smart_transfer(n, dir_step, max_ray, board, plates, ops, p, q, dist_table, ctx);
        if !success {
            break;
        }
    }
}
fn solve_with_stages_advanced(
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
    use_chunk_mega: bool,
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board;
    let mut ops: Vec<Op> = Vec::with_capacity(300);
    let mut init_total_slimes = 0;
    for p in 0..n * n {
        init_total_slimes += init_board.len(p);
    }
    ctx.init_total_slimes = init_total_slimes;
    let floor_cells = floor.iter().filter(|&&f| f).count();
    ctx.density = init_total_slimes as f64 / floor_cells.max(1) as f64;

    // =========================================================
    // Stage 1: 事前クラスタリングフェーズ
    // 0: なし, 1: 局所ペアマージ, 2: フルタワー事前集約
    // =========================================================
    for &color in color_order {
        match stage1_modes[color] {
            1 => {
                let target_plate = plates[color];
                let mut positions = [0usize; 400];
                let mut pos_count = 0;
                for p in 0..n * n {
                    if board.last(p) == Some(color) {
                        positions[pos_count] = p;
                        pos_count += 1;
                    }
                }
                if pos_count >= 2 {
                    positions[..pos_count].sort_by_key(|&p| std::cmp::Reverse(dist_table[p][target_plate]));

                    for i in 0..pos_count {
                        let p = positions[i];
                        if board.last(p) != Some(color) { continue; }
                        let h_p = board.len(p);
                        let d_p_tgt = dist_table[p][target_plate];

                        let mut best_partner = None;
                        let mut min_partner_dist = usize::MAX;

                        for j in 0..pos_count {
                            if i == j { continue; }
                            let q = positions[j];
                            if board.last(q) != Some(color) { continue; }
                            let h_q = board.len(q);
                            if h_p + h_q > 8 { continue; }

                            let d_q_tgt = dist_table[q][target_plate];
                            if d_q_tgt <= d_p_tgt {
                                let d_pq = dist_table[p][q];
                                if d_pq <= 3 && d_pq < min_partner_dist {
                                    min_partner_dist = d_pq;
                                    best_partner = Some(q);
                                }
                            }
                        }

                        if let Some(q) = best_partner {
                            move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, q, dist_table, ctx);
                        }
                    }
                }
            }
            2 => {
                pre_cluster_full_towers(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx);
            }
            3 => {
                pre_cluster_full_towers(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx);
                pre_cluster_multi_color_for_color(n, dir_step, max_ray, &mut board, plates, &mut ops, color, dist_table, ctx);
            }
            _ => {}
        }
    }

    let mut total_slimes = 0;
    for p in 0..n * n {
        total_slimes += board.len(p);
    }
    let wall_cnt = floor.iter().filter(|&&f| !f).count();
    let enable_snowball = total_slimes <= 75 || total_slimes > 115 || wall_cnt < 70;
    if use_chunk_mega && enable_snowball {
        pre_cluster_chunked_mega_towers(n, dir_step, max_ray, &mut board, plates, &mut ops, dist_table, ctx, init_total_slimes);
    }

    // =========================================================

    // Stage 2: 一括マルチ帰巣フェーズ
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

                let total_rem = board.count_color(color);

                // 1. マルチスライム一括ダイレクト射出（直線1手・ボトルネック開放）
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

                // 2. マルチスライム2段跳躍一括帰巣
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

                let mut moved_any = false;

                if strat == 5 {
                    // =========================================================
                    // アプローチ C: 巡回メガタワー (TSP Sweeper)
                    // 巣穴から一番遠いスライムを起点とし、最も近い未回収スライムへ次々と
                    // 訪問・合体しながら巨大タワーを雪だるま式に成長させて最後に巣穴へ帰還！
                    // =========================================================
                    let mut unvisited = group.to_vec();
                    unvisited.sort_by_key(|&p| std::cmp::Reverse(dist_table[p][target_plate]));

                    let mut current_pos = unvisited.remove(0);

                    while !unvisited.is_empty() {
                        let mut best_idx = 0;
                        let mut min_d = usize::MAX;
                        for (idx, &v) in unvisited.iter().enumerate() {
                            let d = dist_table[current_pos][v];
                            if d < min_d {
                                min_d = d;
                                best_idx = idx;
                            }
                        }
                        let next_target = unvisited.remove(best_idx);
                        let d_curr = dist_table[current_pos][target_plate];
                        let d_next = dist_table[next_target][target_plate];
                        let h_curr = board.len(current_pos);
                        let h_next = board.len(next_target);

                        let prefer_next_to_curr = if d_curr < d_next {
                            true
                        } else if d_curr > d_next {
                            false
                        } else {
                            h_curr >= h_next
                        };

                        if prefer_next_to_curr {
                            if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, next_target, current_pos, dist_table, ctx) {
                                moved_any = true;
                            } else if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, current_pos, next_target, dist_table, ctx) {
                                moved_any = true;
                                current_pos = next_target;
                            }
                        } else {
                            if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, current_pos, next_target, dist_table, ctx) {
                                moved_any = true;
                                current_pos = next_target;
                            } else if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, next_target, current_pos, dist_table, ctx) {
                                moved_any = true;
                            }
                        }
                    }

                    if current_pos != target_plate && !board.is_empty(current_pos) {
                        if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, current_pos, target_plate, dist_table, ctx) {
                            moved_any = true;
                        }
                    }
                } else if strat == 4 {
                    // =========================================================
                    // アプローチ B: カタパルト中継砲台 (Catapult Relay)
                    // 巣穴への射線（直線レイ）が直接通るマスをカタパルト Hub に選定し、
                    // 集約した高層タワーから巣穴へ 1手大ジャンプで射出する！
                    // =========================================================
                    let r_target = target_plate / n;
                    let c_target = target_plate % n;
                    let mut best_catapult = target_plate;
                    let mut min_catapult_cost = usize::MAX;

                    for x in 0..n * n {
                        if !floor[x] || (!board.is_empty(x) && !group.contains(&x) && x != target_plate) {
                            continue;
                        }
                        if dist_table[x][target_plate] == usize::MAX {
                            continue;
                        }
                        let r_x = x / n;
                        let c_x = x % n;

                        let mut direct_ray = false;
                        if r_x == r_target {
                            let c_min = c_x.min(c_target);
                            let c_max = c_x.max(c_target);
                            let mut clear = true;
                            for c in c_min..=c_max {
                                if !floor[r_x * n + c] { clear = false; break; }
                            }
                            if clear { direct_ray = true; }
                        } else if c_x == c_target {
                            let r_min = r_x.min(r_target);
                            let r_max = r_x.max(r_target);
                            let mut clear = true;
                            for r in r_min..=r_max {
                                if !floor[r * n + c_x] { clear = false; break; }
                            }
                            if clear { direct_ray = true; }
                        }

                        let d_nest = dist_table[x][target_plate];
                        let mut cost = d_nest;
                        if direct_ray && d_nest <= 8 {
                            cost = cost.saturating_sub(10);
                        } else if direct_ray {
                            cost = cost.saturating_sub(4);
                        }

                        let mut possible = true;
                        for &p in group {
                            if dist_table[p][x] == usize::MAX {
                                possible = false;
                                break;
                            }
                            cost += dist_table[p][x];
                        }

                        if possible && cost < min_catapult_cost {
                            min_catapult_cost = cost;
                            best_catapult = x;
                        }
                    }
                    best_x = best_catapult;

                    let mut sorted_group = group.to_vec();
                    sorted_group.sort_by_key(|&p| dist_table[p][best_x]);

                    for &p in &sorted_group {
                        if p == best_x {
                            continue;
                        }
                        let success = move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, best_x, dist_table, ctx);
                        if success {
                            moved_any = true;
                        } else if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, dist_table, ctx) {
                            moved_any = true;
                        }
                    }

                    if best_x != target_plate && !board.is_empty(best_x) {
                        if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, dist_table, ctx) {
                            moved_any = true;
                        }
                    }
                } else if strat == 3 {
                    // =========================================================
                    // アプローチ A: 樹状合流ネットワーク (Tree-based Aggregation)
                    // 末端の葉ノードから順に、親ノードへ合流させてタワーを太くしながら進む
                    // =========================================================
                    let mut min_cost = usize::MAX;
                    let r_target = target_plate / n;
                    let c_target = target_plate % n;

                    for x in 0..n * n {
                        if !floor[x] || (!board.is_empty(x) && !group.contains(&x) && x != target_plate) {
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
                                cost = cost.saturating_sub(1);
                            }

                            // 他色タワー隣接・トランポリンジャンプボーナス！
                            for d in 0..4 {
                                if max_ray[x][d] >= 1 {
                                    let adj_p = (x as isize + dir_step[d]) as usize;
                                    let adj_len = board.len(adj_p);
                                    if adj_len >= 2 && !group.contains(&adj_p) && adj_len + group.len() <= 8 {
                                        cost = cost.saturating_sub(2);
                                        break;
                                    }
                                }
                            }

                            if cost < min_cost {
                                min_cost = cost;
                                best_x = x;
                            }
                        }
                    }

                    let root = best_x;
                    let mut tree_nodes = group.to_vec();
                    tree_nodes.sort_by_key(|&p| std::cmp::Reverse(dist_table[p][root]));

                    let mut moved_in_tree = false;
                    for i in 0..tree_nodes.len() {
                        let u = tree_nodes[i];
                        if u == root || board.len(u) == 0 { continue; }

                        let d_u_root = dist_table[u][root];
                        let mut best_parent = root;
                        let mut min_dist_to_parent = dist_table[u][root];

                        for j in (i + 1)..tree_nodes.len() {
                            let v = tree_nodes[j];
                            if board.len(v) == 0 || board.len(u) + board.len(v) > 8 { continue; }
                            let d_v_root = dist_table[v][root];
                            if d_v_root < d_u_root {
                                let d_uv = dist_table[u][v];
                                if d_uv < min_dist_to_parent {
                                    min_dist_to_parent = d_uv;
                                    best_parent = v;
                                }
                            }
                        }

                        let success = move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, u, best_parent, dist_table, ctx);
                        if success {
                            moved_in_tree = true;
                            moved_any = true;
                        } else if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, u, root, dist_table, ctx) {
                            moved_in_tree = true;
                            moved_any = true;
                        }
                    }
                    if !moved_in_tree {
                        for &p in group {
                            if p == best_x { continue; }
                            if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, best_x, dist_table, ctx) {
                                moved_any = true;
                            }
                        }
                    }

                    if best_x != target_plate && !board.is_empty(best_x) {
                        if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, dist_table, ctx) {
                            moved_any = true;
                        }
                    }
                } else {
                    // strat == 0, 1, 2
                    if strat == 2 {
                        best_x = target_plate;
                    } else {
                        let mut min_cost = usize::MAX;
                        let r_target = target_plate / n;
                        let c_target = target_plate % n;

                        for x in 0..n * n {
                            if !floor[x] || (!board.is_empty(x) && !group.contains(&x) && x != target_plate) {
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
                                    cost = cost.saturating_sub(1);
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
                        let success = move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, best_x, dist_table, ctx);
                        if success {
                            moved_any = true;
                        } else if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, dist_table, ctx) {
                            moved_any = true;
                        }
                    }

                    if best_x != target_plate && !board.is_empty(best_x) {
                        if move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, best_x, target_plate, dist_table, ctx) {
                            moved_any = true;
                        }
                    }
                }

                eject_nest_blockers_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, target_plate, color, dist_table);

                // Step 38: 盤面の異色混合タワーから大ジャンプ射出・デポ！
                for p in 0..n * n {
                    if board.len(p) >= 2 {
                        try_cross_color_ejection_fast(n, dir_step, max_ray, &mut board, plates, &mut ops, p, dist_table);
                    }
                }

                if !moved_any {
                    // Cannot move any slimes of this color, don't spin for 50 iterations!
                    break;
                }
            }
        }

        if !progress {
            for p in 0..n * n {
                if let Some(color) = board.last(p) {
                    let target_plate = plates[color];
                    move_smart_transfer(n, dir_step, max_ray, &mut board, plates, &mut ops, p, target_plate, dist_table, ctx);
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

pub fn pre_cluster_dense(
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
    let target_plate = plates[color];
    let mut positions = Vec::new();
    for p in 0..n * n {
        if board.last(p) == Some(color) {
            positions.push(p);
        }
    }
    if positions.len() < 2 {
        return;
    }

    positions.sort_by_key(|&p| std::cmp::Reverse(dist_table[p][target_plate]));

    for i in 0..positions.len() {
        if ops.len() > 60 { break; }
        let u = positions[i];
        if u == target_plate || board.len(u) == 0 { continue; }
        let d_u_root = dist_table[u][target_plate];
        if d_u_root == usize::MAX { continue; }

        let mut best_parent = target_plate;
        let mut min_dist_to_parent = usize::MAX;

        for j in (i + 1)..positions.len() {
            let v = positions[j];
            if board.len(v) == 0 || board.len(u) + board.len(v) > 4 { continue; }
            let d_v_root = dist_table[v][target_plate];
            if d_v_root < d_u_root {
                let d_uv = dist_table[u][v];
                if d_uv <= 2 && d_uv < min_dist_to_parent {
                    min_dist_to_parent = d_uv;
                    best_parent = v;
                }
            }
        }

        if best_parent != target_plate && min_dist_to_parent <= 2 {
            move_smart_transfer(n, dir_step, max_ray, board, plates, ops, u, best_parent, dist_table, ctx);
        }
    }
}

pub fn solve(input_str: &str, params: &crate::TunableParams) {
    let start_time = Instant::now();

    let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());

    let first_line = lines.next().unwrap().trim_start_matches('\u{feff}');
    let mut parts = first_line.split_whitespace();
    let n: usize = parts.next().unwrap().parse().unwrap();
    let k: usize = parts.next().unwrap().parse().unwrap();

    let mut floor = vec![true; n * n];
    let mut init_stacks: Vec<Vec<usize>> = vec![Vec::new(); n * n];
    let mut plates = vec![0; k];
    let mut raw_grid = Vec::with_capacity(n);

    for i in 0..n {
        let line = lines.next().unwrap();
        let chars: Vec<char> = line.chars().collect();
        for (j, &ch) in chars.iter().enumerate() {
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
        raw_grid.push(chars);
    }

    let dir_step: [isize; 4] = [-(n as isize), n as isize, -1, 1];
    let max_ray = compute_max_ray(n, &floor);
    let dist_table = compute_all_pairs_dist(n, &floor);
    let init_board = BitBoard::new(n, &init_stacks);
    let wall_count = floor.iter().filter(|&&f| !f).count();
    let mut ctx = Box::new(BfsContext::new());
    ctx.allow_01_detour = wall_count <= 90;

    // ---------------------------------------------------------
    // Phase 1: 高速初期スクリーニング（複数ヒューリスティック順序評価: ~0.005s）
    // ---------------------------------------------------------
    let mut rng = XorShift::new(20260925);
    let mut best_init_order = (0..k).collect::<Vec<_>>();
    let mut best_init_score = usize::MAX;
    let mut best_init_ops = Vec::new();
    let mut initial_slime_count = 0;
    for p in 0..n * n {
        initial_slime_count += init_board.len(p);
    }
    ctx.init_total_slimes = initial_slime_count;
    let floor_cells = floor.iter().filter(|&&f| f).count();
    ctx.density = initial_slime_count as f64 / floor_cells.max(1) as f64;
    let default_group_sizes = if initial_slime_count >= 100 {
        [8usize; 16]
    } else {
        [6usize; 16]
    };
    let default_stage1_modes = [2u8; 16];

    let mut order_candidates = Vec::new();
    order_candidates.push((0..k).collect::<Vec<_>>());
    let mut rev_order = (0..k).collect::<Vec<_>>();
    rev_order.reverse();
    order_candidates.push(rev_order);

    let mut color_dists = Vec::new();
    for c in 0..k {
        let tgt = plates[c];
        let mut d_sum = 0;
        let mut cnt = 0;
        for p in 0..n * n {
            if init_board.contains(p, c) {
                let d = dist_table[p][tgt];
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

    for _ in 0..6 {
        let mut shuf = (0..k).collect::<Vec<_>>();
        for i in (1..k).rev() {
            let j = rng.gen_range(0, i + 1);
            shuf.swap(i, j);
        }
        order_candidates.push(shuf);
    }

    let mut default_stage1_modes_actual = default_stage1_modes;
    let mut best_use_chunk = false;
    let mut best_strategies = [3u8; 16]; // デフォルトで最強の樹状合流 (Approach A)

    let enable_snowball = initial_slime_count <= 75 || initial_slime_count > 115 || wall_count < 70;
    let chunk_choices = if enable_snowball { &[false, true][..] } else { &[false][..] };
    // Dense 帯 (> 115) 限定で Strategy 4 (Catapult Relay) を Phase 1 スクリーニングに追加
    // Small/Medium 帯では樹状合流の基本ツリー構造を維持して局所解トラップを防止
    let strat_choices_phase1: &[u8] = if initial_slime_count > 115 {
        &[3u8, 5u8, 4u8, 0u8]
    } else {
        &[3u8, 5u8, 0u8]
    };
    for order in &order_candidates {
        for &use_chunk in chunk_choices {
            for &strat_mode in strat_choices_phase1 {
                let strats = [strat_mode; 16];
                let (score, ops) = solve_with_stages_advanced(
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
                    &strats,
                    &default_stage1_modes,
                    use_chunk,
                    &mut ctx,
                );
                if score < best_init_score {
                    best_init_score = score;
                    best_init_order = order.clone();
                    best_init_ops = ops;
                    default_stage1_modes_actual = default_stage1_modes;
                    best_use_chunk = use_chunk;
                    best_strategies = strats;
                }
            }
        }
    }

    // Courier Solo モードのスクリーニング:
    // best_init_order に対して、先頭色のみ集約し他色を温存する Courier Solo を評価
    // 従来の best_init_score より真に優れている場合のみ採択！
    {
        let mut modes_solo = [0u8; 16];
        modes_solo[best_init_order[0]] = 2;
        let (score_solo, ops_solo) = solve_with_stages_advanced(
            n,
            k,
            &dir_step,
            &max_ray,
            &floor,
            &init_board,
            &plates,
            &dist_table,
            &best_init_order,
            &default_group_sizes,
            &best_strategies,
            &modes_solo,
            best_use_chunk,
            &mut ctx,
        );
        if score_solo < best_init_score {
            best_init_score = score_solo;
            best_init_ops = ops_solo;
            default_stage1_modes_actual = modes_solo;
        }
    }

    // 局所ペアマージ（Local Pairwise Merge）のスクリーニング評価（Dense帯かつ壁が多すぎない盤面でのみ評価！）
    if initial_slime_count > 115 && wall_count < 80 {
        let modes_local = [1u8; 16];
        let (score_local, ops_local) = solve_with_stages_advanced(
            n,
            k,
            &dir_step,
            &max_ray,
            &floor,
            &init_board,
            &plates,
            &dist_table,
            &best_init_order,
            &default_group_sizes,
            &best_strategies,
            &modes_local,
            best_use_chunk,
            &mut ctx,
        );
        if score_local < best_init_score {
            best_init_score = score_local;
            best_init_ops = ops_local;
            default_stage1_modes_actual = modes_local;
        }
    }

    // ---------------------------------------------------------
    // Phase 2: 焼きなまし法（SA: 1400ms、後処理に 350ms、1750ms で強制打切り）
    // ---------------------------------------------------------
    let total_time_limit = std::time::Duration::from_millis(1400);
    let mut current_order = best_init_order;
    let mut current_group_sizes = default_group_sizes;
    let mut current_strategies = best_strategies;
    let mut current_stage1_modes = default_stage1_modes_actual;
    let mut current_use_chunk = best_use_chunk;
    let mut current_score = best_init_score;
    let mut sa_best_score = best_init_score;
    let mut sa_best_ops = best_init_ops.clone();
    let mut sa_best_order = current_order.clone();
    let mut sa_best_strategies = current_strategies;
    let mut sa_best_stage1_modes = current_stage1_modes;
    let mut _sa_best_use_chunk = current_use_chunk;

    let t_start: f64 = params.t_start;
    let t_end: f64 = params.t_end;
    let mut sa_iter_count = 0;

    // 動的戦略候補配列の構築
    let mut strat_choices = Vec::new();
    for _ in 0..params.w_strat_tree { strat_choices.push(3u8); }
    for _ in 0..params.w_strat_sweep { strat_choices.push(5u8); }
    for _ in 0..params.w_strat_catapult { strat_choices.push(4u8); }
    for _ in 0..params.w_strat_centroid { strat_choices.push(0u8); }
    strat_choices.push(1u8);
    strat_choices.push(2u8);
    if strat_choices.is_empty() { strat_choices.push(3u8); }

    let th_strat = 70 + params.p_strategy.min(25);

    let mut candidate_pools: Vec<Vec<Op>> = Vec::new();
    candidate_pools.push(best_init_ops.clone());

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
        let mut next_use_chunk = current_use_chunk;

        let move_type = rng.gen_range(0, 100);
        if move_type < 20 {
            let i = rng.gen_range(0, k);
            let j = rng.gen_range(0, k);
            next_order.swap(i, j);
        } else if move_type < 40 {
            let from = rng.gen_range(0, k);
            let to = rng.gen_range(0, k);
            let val = next_order.remove(from);
            next_order.insert(to, val);
        } else if move_type < 55 {
            let mut i = rng.gen_range(0, k);
            let mut j = rng.gen_range(0, k);
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            next_order[i..=j].reverse();
        } else if move_type < 70 {
            let c = rng.gen_range(0, k);
            next_group_sizes[c] = rng.gen_range(4, 9);
        } else if move_type < th_strat {
            let c = rng.gen_range(0, k);
            next_strategies[c] = strat_choices[rng.gen_range(0, strat_choices.len())];
        } else if move_type < 93 {
            let c = rng.gen_range(0, k);
            next_stage1_modes[c] = rng.gen_range(0, 4) as u8;
        } else if move_type < 97 {
            if enable_snowball {
                next_use_chunk = !current_use_chunk;
            }
        } else {
            let sub = rng.gen_range(0, 3);
            match sub {
                0 => {
                    let lead = next_order[0];
                    next_stage1_modes = [0u8; 16];
                    next_stage1_modes[lead] = 2;
                }
                1 => {
                    next_stage1_modes = [2u8; 16];
                }
                _ => {
                    let lead = next_order[0];
                    next_stage1_modes[lead] = rng.gen_range(0, 4) as u8;
                }
            }
        }

        let (cand_score, cand_ops) = solve_with_stages_advanced(
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
            next_use_chunk,
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
            current_use_chunk = next_use_chunk;
            current_score = cand_score;

            if cand_score < sa_best_score {
                sa_best_score = cand_score;
                sa_best_ops = cand_ops.clone();
                sa_best_order = current_order.clone();
                sa_best_strategies = current_strategies;
                sa_best_stage1_modes = current_stage1_modes;
                _sa_best_use_chunk = current_use_chunk;
            }

            // 有望候補プールへの追加（スコアが最良に近い多様な解）
            if cand_score <= sa_best_score + 8 {
                if candidate_pools.len() < 6 {
                    if !candidate_pools.iter().any(|c| c.len() == cand_ops.len()) {
                        candidate_pools.push(cand_ops);
                    }
                } else {
                    let max_idx = candidate_pools.iter().enumerate().max_by_key(|(_, c)| c.len()).map(|(i, _)| i).unwrap();
                    if cand_ops.len() < candidate_pools[max_idx].len() && !candidate_pools.iter().any(|c| c.len() == cand_ops.len()) {
                        candidate_pools[max_idx] = cand_ops;
                    }
                }
            }
        }
    }

    if !candidate_pools.iter().any(|c| c.len() == sa_best_ops.len()) {
        candidate_pools.push(sa_best_ops.clone());
    }

    let (b01_grid, b01_board) = crate::b01::Grid::new(n, k, &raw_grid);

    // 短縮パイプラインヘルパー関数
    let run_pipeline = |ops_a44: Vec<Op>| -> Vec<crate::b01::Op> {
        let ops_b01: Vec<crate::b01::Op> = ops_a44
            .into_iter()
            .map(|op| {
                let d = match op.d {
                    'U' => 0,
                    'D' => 1,
                    'L' => 2,
                    'R' => 3,
                    _ => 0,
                };
                crate::b01::Op {
                    r: op.r,
                    c: op.c,
                    k: op.k,
                    d,
                    l: op.l,
                }
            })
            .collect();

        let mode = crate::b01::post_mode();
        let total_begin = Instant::now();
        let raw_len = ops_b01.len();
        let total_deadline_ms: u128 = if mode == "fast" { 1750 } else { 1650 };

        let run_opt_step = |mut ops: Vec<crate::b01::Op>, opt_fn: fn(Vec<crate::b01::Op>, &crate::b01::BitBoard, &crate::b01::Grid, usize, Instant, u128) -> Vec<crate::b01::Op>| -> Vec<crate::b01::Op> {
            if start_time.elapsed().as_millis() >= total_deadline_ms {
                return ops;
            }
            let prev = ops.len();
            ops = opt_fn(ops, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
            if ops.len() < prev && start_time.elapsed().as_millis() < total_deadline_ms {
                ops = crate::b01::prune_ops(ops, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
            }
            ops
        };

        let mut cur_ops = crate::b01::prune_ops(ops_b01, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
        cur_ops = crate::b01::prune_cycle_and_uturn_ops(cur_ops, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
        for _ in 0..2 {
            if start_time.elapsed().as_millis() >= total_deadline_ms {
                break;
            }
            let before_len = cur_ops.len();
            cur_ops = run_opt_step(cur_ops, crate::b01::merge_straight_jumps);
            cur_ops = run_opt_step(cur_ops, crate::b01::shortcut_ray_jumps);
            cur_ops = run_opt_step(cur_ops, crate::b01::orthogonal_2hop_jumps);
            cur_ops = run_opt_step(cur_ops, crate::b01::coriding_leap_jumps);
            cur_ops = run_opt_step(cur_ops, crate::b01::crank_3hop_jumps);
            cur_ops = crate::b01::prune_cycle_and_uturn_ops(cur_ops, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
            if cur_ops.len() == before_len {
                break;
            }
        }
        eprintln!("PHASE: name=geometry elapsed_ms={:.3} before={} after={}", total_begin.elapsed().as_secs_f64()*1000.0, raw_len, cur_ops.len());
        if mode != "fast" {
            let final_ops = crate::b01::finish_new_pipeline(cur_ops, &b01_board, &b01_grid, start_time, &mode);
            eprintln!("CANDIDATE: raw={} final={}", raw_len, final_ops.len());
            return final_ops;
        }
        let ops_lns = if start_time.elapsed().as_millis() + 150 < total_deadline_ms {
            crate::b01::lns_optimize(cur_ops, &b01_board, &b01_grid, start_time, total_deadline_ms)
        } else {
            cur_ops
        };
        let mut final_cleaned_ops = crate::b01::prune_ops(ops_lns, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
        final_cleaned_ops = crate::b01::prune_cycle_and_uturn_ops(final_cleaned_ops, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
        for _ in 0..2 {
            if start_time.elapsed().as_millis() >= total_deadline_ms {
                break;
            }
            let before_len = final_cleaned_ops.len();
            final_cleaned_ops = run_opt_step(final_cleaned_ops, crate::b01::merge_straight_jumps);
            final_cleaned_ops = run_opt_step(final_cleaned_ops, crate::b01::shortcut_ray_jumps);
            final_cleaned_ops = run_opt_step(final_cleaned_ops, crate::b01::orthogonal_2hop_jumps);
            final_cleaned_ops = run_opt_step(final_cleaned_ops, crate::b01::coriding_leap_jumps);
            final_cleaned_ops = run_opt_step(final_cleaned_ops, crate::b01::crank_3hop_jumps);
            final_cleaned_ops = crate::b01::prune_cycle_and_uturn_ops(final_cleaned_ops, &b01_board, &b01_grid, b01_grid.initial_slimes, start_time, total_deadline_ms);
            if final_cleaned_ops.len() == before_len {
                break;
            }
        }
        final_cleaned_ops
    };

    // 全候補プールに対して後処理を実行し、真の確定最小手数の解を選択
    candidate_pools.sort_by_key(|c| c.len());
    let fallback: Vec<crate::b01::Op> = sa_best_ops.iter().map(|op| crate::b01::Op {
        r:op.r,c:op.c,k:op.k,d:match op.d {'U'=>0,'D'=>1,'L'=>2,_=>3},l:op.l,
    }).collect();
    let mut best_final_ops = if crate::b01::valid_complete(&b01_board,&fallback,&b01_grid) {
        Some(fallback)
    } else { None };
    let pool_candidates = candidate_pools.len();
    let mut processed_candidates = 0;
    for cand_ops in candidate_pools {
        if start_time.elapsed().as_millis() >= 1650 {
            break;
        }
        let cleaned = run_pipeline(cand_ops);
        processed_candidates += 1;
        if crate::b01::valid_complete(&b01_board,&cleaned,&b01_grid) && best_final_ops.as_ref().map_or(true, |b| cleaned.len() < b.len()) {
            best_final_ops = Some(cleaned);
        }
    }
    let final_cleaned_ops = best_final_ops.expect("no validated candidate");
    eprintln!("POOL: raw_best_ops={} pool_candidates={} processed_candidates={}",sa_best_ops.len(),pool_candidates,processed_candidates);

    let winner = "SimulatedAnnealing+Prune";
    eprintln!(
        "STAT: beam_iters=0 sa_iters={} beam_score=0 sa_score={} winner={} final_score={} best_order={:?} best_strats={:?} best_modes={:?}",
        sa_iter_count, sa_best_score, winner, final_cleaned_ops.len(), sa_best_order, &sa_best_strategies[..k], &sa_best_stage1_modes[..k]
    );

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &final_cleaned_ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, crate::b01::DIR_CHAR[op.d], op.l).unwrap();
    }
}

}


mod b01 {
// AHC 072 - B_01: 極限高速化 ＆ 100%完全帰巣保証ハイブリッド
//
// 1. 完全ゼロアロケーション探索ループ:
//    毎手の Vec 確保を完全撤廃し、再利用バッファでヒープアロケーションをゼロ化。
// 2. プレフィルタ付き二分探索ビーム挿入:
//    ビーム最下位スコア以下の候補を即座に破棄し、無駄な配列シフトを90%削減。
// 3. 完全差分評価 & 差分 Zobrist ハッシュ:
//    動いた2マスのみを O(1) で加減算更新。
// 4. ビットマスク走査 (trailing_zeros):
//    スライム存在マスのみを高速ピンポイント列挙。
// 5. 100%完全クリア保証フォールバック (Greedy Clean-up):
//    Chokudai Search 終了時点で万一スライムが残っていた場合、
//    最良盤面から最短経路BFS誘導で残存スライムを確実に全頭巣へ収容（E=0 を完全保証）。

use std::hash::{BuildHasherDefault, Hasher};
// use std::io::{self, Read};
use std::time::Instant;

const DIR: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
pub const DIR_CHAR: [char; 4] = ['U', 'D', 'L', 'R'];
const OPPOSITE_DIR: [usize; 4] = [1, 0, 3, 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Op {
    pub r: usize,
    pub c: usize,
    pub k: usize,
    pub d: usize, // 0: U, 1: D, 2: L, 3: R
    pub l: usize,
}

// -------------------------------------------------------------
// 高速ハッシャー（軽量乗算ハッシュ）
// -------------------------------------------------------------
#[derive(Default)]
struct FastHasher(u64);

impl Hasher for FastHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = (self.0 ^ i).wrapping_mul(0x517cc1b727220a95);
    }
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ (b as u64)).wrapping_mul(0x517cc1b727220a95);
        }
    }
}

type FastHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FastHasher>>;

// -------------------------------------------------------------
// 高速盤面表現 BitBoard (最大 20x20 = 400 マス)
// -------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BitBoard {
    pub colors: [u32; 400],
    pub lens: [u8; 400],
    pub occupied_mask: [u64; 7],
}

impl BitBoard {
    fn new() -> Self {
        Self {
            colors: [0; 400],
            lens: [0; 400],
            occupied_mask: [0; 7],
        }
    }

    #[inline(always)]
    fn set_occupied(&mut self, p: usize) {
        self.occupied_mask[p / 64] |= 1u64 << (p % 64);
    }

    #[inline(always)]
    fn clear_occupied(&mut self, p: usize) {
        self.occupied_mask[p / 64] &= !(1u64 << (p % 64));
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
    fn clean_nest(&mut self, p: usize, nest_color: usize) -> usize {
        let mut cleaned = 0;
        while self.lens[p] > 0 {
            let shift = (self.lens[p] as usize - 1) * 4;
            if ((self.colors[p] >> shift) & 0xF) as usize == nest_color {
                self.lens[p] -= 1;
                let new_l = self.lens[p] as usize;
                self.colors[p] &= if new_l == 0 {
                    0
                } else {
                    (1u32 << (new_l * 4)) - 1
                };
                cleaned += 1;
            } else {
                break;
            }
        }
        cleaned
    }
}

// -------------------------------------------------------------
// グリッド・定数情報
// -------------------------------------------------------------
#[allow(dead_code)]
pub struct Grid {
    pub n: usize,
    pub k: usize,
    pub cell_count: usize,
    pub is_floor: [bool; 400],
    pub nest_pos: [usize; 16],
    pub nest_color_at: [Option<usize>; 400],
    pub dist: Vec<u16>, // 400 * 400
    pub max_reach: [[usize; 4]; 400], // 各マスから各方向に進める最大直線距離
    pub initial_slimes: usize,
    pub zobrist: Vec<u64>, // 400 * 8 * 16
    pub line_of_sight: Vec<bool>, // 400 * 16: 直線上に巣があるか
}

impl Grid {
    pub fn new(n: usize, k: usize, raw_grid: &[Vec<char>]) -> (Self, BitBoard) {
        let cell_count = n * n;
        let mut is_floor = [false; 400];
        let mut nest_pos = [0; 16];
        let mut nest_color_at = [None; 400];
        let mut board = BitBoard::new();
        let mut initial_slimes = 0;

        for r in 0..n {
            for c in 0..n {
                let p = r * n + c;
                let ch = raw_grid[r][c];
                if ch != '#' {
                    is_floor[p] = true;
                }
                if ch >= 'A' && ch <= 'L' {
                    let col = (ch as u8 - b'A') as usize;
                    nest_pos[col] = p;
                    nest_color_at[p] = Some(col);
                } else if ch >= 'a' && ch <= 'l' {
                    let col = (ch as u8 - b'a') as usize;
                    board.lens[p] = 1;
                    board.colors[p] = col as u32;
                    board.set_occupied(p);
                    initial_slimes += 1;
                }
            }
        }

        // 全マス間の床最短距離 BFS
        let mut dist = vec![9999u16; 400 * 400];
        for start in 0..cell_count {
            if !is_floor[start] {
                continue;
            }
            dist[start * 400 + start] = 0;
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);
            while let Some(curr) = queue.pop_front() {
                let d = dist[start * 400 + curr];
                let cr = curr / n;
                let cc = curr % n;
                for &(dr, dc) in &DIR {
                    let nr = cr as isize + dr;
                    let nc = cc as isize + dc;
                    if nr >= 0 && nr < n as isize && nc >= 0 && nc < n as isize {
                        let next_p = nr as usize * n + nc as usize;
                        if is_floor[next_p] && dist[start * 400 + next_p] > d + 1 {
                            dist[start * 400 + next_p] = d + 1;
                            queue.push_back(next_p);
                        }
                    }
                }
            }
        }

        // 各マスの最大移動可能距離（途中に壁がない最大直線距離）
        let mut max_reach = [[0usize; 4]; 400];
        for p in 0..cell_count {
            if !is_floor[p] {
                continue;
            }
            let r = p / n;
            let c = p % n;
            for d in 0..4 {
                let (dr, dc) = DIR[d];
                let mut step = 0;
                loop {
                    let nr = r as isize + dr * (step as isize + 1);
                    let nc = c as isize + dc * (step as isize + 1);
                    if nr >= 0 && nr < n as isize && nc >= 0 && nc < n as isize {
                        let next_p = nr as usize * n + nc as usize;
                        if is_floor[next_p] {
                            step += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                max_reach[p][d] = step;
            }
        }

        // Zobrist テーブル生成（XorShift による疑似乱数）
        let mut rng = 88172645463325252u64;
        let mut zobrist = vec![0u64; 400 * 8 * 16];
        for p in 0..cell_count {
            for h in 0..8 {
                for c in 0..16 {
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    zobrist[(p * 8 + h) * 16 + c] = rng;
                }
            }
        }

        // 巣への直通ライン事前計算（マス p から色 col の巣へ途中に壁がなく直通しているか）
        let mut line_of_sight = vec![false; 400 * 16];
        for p in 0..cell_count {
            if !is_floor[p] {
                continue;
            }
            let r = p / n;
            let c = p % n;
            for col in 0..k {
                let nest = nest_pos[col];
                let nr = nest / n;
                let nc = nest % n;
                if r == nr && c != nc {
                    let d = if nc > c { 1 } else { 3 }; // 1: 右, 3: 左
                    let dist = (nc as isize - c as isize).abs() as usize;
                    if dist <= max_reach[p][d] {
                        line_of_sight[p * 16 + col] = true;
                    }
                } else if c == nc && r != nr {
                    let d = if nr > r { 2 } else { 0 }; // 2: 下, 0: 上
                    let dist = (nr as isize - r as isize).abs() as usize;
                    if dist <= max_reach[p][d] {
                        line_of_sight[p * 16 + col] = true;
                    }
                }
            }
        }

        (
            Self {
                n,
                k,
                cell_count,
                is_floor,
                nest_pos,
                nest_color_at,
                dist,
                max_reach,
                initial_slimes,
                zobrist,
                line_of_sight,
            },
            board,
        )
    }

    #[inline(always)]
    fn get_dist(&self, p1: usize, p2: usize) -> u16 {
        self.dist[p1 * 400 + p2]
    }

    #[inline(always)]
    fn get_zobrist(&self, p: usize, h: usize, col: usize) -> u64 {
        self.zobrist[(p * 8 + h) * 16 + col]
    }

    #[inline(always)]
    fn has_line_of_sight(&self, p: usize, col: usize) -> bool {
        self.line_of_sight[p * 16 + col]
    }
}

// -------------------------------------------------------------
// ハイパーパラメータ
// -------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub w_tower_quad: i64,
    pub w_tower_linear: i64,
    pub w_pair: i64,
    pub w_nest: i64,
    pub w_purity: i64,
    pub w_mix: i64,
    pub w_depth: i64,
    pub w_line_of_sight: i64, // 【新】巣への直通ラインボーナス
    pub w_top_match: i64,     // 【新】トップの即帰巣可能ボーナス
    pub w_nest_block: i64,    // 【新】巣の上の異色スライム立ち退きペナルティ
    pub w_color_run_quad: i64,   // 【新】同色連続長 B に対する 2乗ボーナス (B^2 * w)
    pub w_color_transition: i64, // 【新】色切り替え回数ペナルティ
    pub w_pure_tower: i64,       // 【新】完全単色タワーボーナス (len^2 * w)
    pub beam_width: usize,
    pub max_cand: usize,
    pub max_depth: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            w_tower_quad: 450,
            w_tower_linear: 900,
            w_pair: 150,
            w_nest: 1500,
            w_purity: 1600,
            w_mix: 2100,
            w_depth: 10,
            w_line_of_sight: 1000,
            w_top_match: 1800,
            w_nest_block: 1800,
            w_color_run_quad: 50,    // 同色連続長 B に対する 2乗ボーナス
            w_color_transition: 200, // 色切り替え回数ペナルティ
            w_pure_tower: 90,        // 完全単色タワーボーナス
            beam_width: 800, // ビーム保持枠 800（キュー枯渇防止・時間いっぱい探索）
            max_cand: 20,   // 候補手数 20
            max_depth: 2000, // 最大深さ 2000
        }
    }
}

// -------------------------------------------------------------
// 1マス単位の静的評価関数 ＆ ハッシュ関数（差分計算用）
// -------------------------------------------------------------
#[inline(always)]
fn eval_cell(colors: u32, len: usize, p: usize, grid: &Grid, params: &Params) -> i64 {
    if len == 0 {
        return 0;
    }
    let mut score = 0i64;

    let l_i = len as i64;
    if len >= 2 {
        score += l_i * l_i * params.w_tower_quad + l_i * params.w_tower_linear;
    }

    score -= params.w_pair * 10;

    let mut c = colors;
    let mut prev_color = 999;
    let mut same_adjacent = 0;
    let mut color_mask = 0u32;
    let mut sum_nest_dist = 0i64;
    let mut line_of_sight_cnt = 0i64;

    let mut run_score = 0i64;
    let mut cur_run = 0i64;
    let mut transitions = 0i64;

    for i in 0..len {
        let col = (c & 0xF) as usize;
        let d_nest = grid.get_dist(p, grid.nest_pos[col]);
        sum_nest_dist += d_nest as i64;

        if grid.has_line_of_sight(p, col) {
            line_of_sight_cnt += 1;
        }

        if i == 0 {
            cur_run = 1;
            prev_color = col;
        } else if col == prev_color {
            cur_run += 1;
            same_adjacent += 1;
        } else {
            if cur_run >= 2 {
                run_score += cur_run * cur_run * params.w_color_run_quad;
            }
            transitions += 1;
            cur_run = 1;
            prev_color = col;
        }

        color_mask |= 1 << col;
        c >>= 4;
    }
    if cur_run >= 2 {
        run_score += cur_run * cur_run * params.w_color_run_quad;
    }

    score -= sum_nest_dist * params.w_nest;
    score += (same_adjacent as i64) * params.w_purity;
    score += run_score;
    score -= transitions * params.w_color_transition;

    // 完全単色タワーボーナス（切り替え 0 回、高さ >= 2）
    if transitions == 0 && len >= 2 {
        score += l_i * l_i * params.w_pure_tower;
    }

    score += line_of_sight_cnt * params.w_line_of_sight;

    // トップのスライムが巣色に合致し、かつ大ジャンプで直接届くか
    let top_col = ((colors >> ((len - 1) * 4)) & 0xF) as usize;
    let top_nest_dist = grid.get_dist(p, grid.nest_pos[top_col]) as usize;
    if grid.has_line_of_sight(p, top_col) && top_nest_dist <= len {
        score += params.w_top_match;
    }

    // 巣の上の異色スライム立ち退きペナルティ
    if let Some(nest_col) = grid.nest_color_at[p] {
        let bottom_col = (colors & 0xF) as usize;
        if bottom_col != nest_col {
            score -= params.w_nest_block;
        }
    }

    let unique_colors = color_mask.count_ones() as i64;
    if unique_colors > 1 {
        score -= (unique_colors - 1) * params.w_mix;
    }

    score
}

#[inline(always)]
fn hash_cell(colors: u32, len: usize, p: usize, grid: &Grid) -> u64 {
    let mut h = 0u64;
    let mut c = colors;
    for height in 0..len {
        let col = (c & 0xF) as usize;
        h ^= grid.get_zobrist(p, height, col);
        c >>= 4;
    }
    h
}

fn compute_full_eval(board: &BitBoard, grid: &Grid, params: &Params) -> (i64, u64) {
    let mut static_score = 0i64;
    let mut hash = 0u64;
    for p in 0..grid.cell_count {
        let len = board.lens[p] as usize;
        if len > 0 {
            static_score += eval_cell(board.colors[p], len, p, grid, params);
            hash ^= hash_cell(board.colors[p], len, p, grid);
        }
    }
    (static_score, hash)
}

// -------------------------------------------------------------
// 高速合法手生成（ゼロアロケーション・バッファ再利用）
// -------------------------------------------------------------
#[inline(always)]
fn generate_valid_moves_into(
    board: &BitBoard,
    last_op: Option<Op>,
    grid: &Grid,
    moves: &mut Vec<Op>,
) {
    moves.clear();

    for word_idx in 0..7 {
        let mut word = board.occupied_mask[word_idx];
        while word != 0 {
            let bit = word.trailing_zeros() as usize;
            let p = word_idx * 64 + bit;
            if p >= grid.cell_count {
                break;
            }

            let h = board.lens[p] as usize;
            let r = p / grid.n;
            let c = p % grid.n;

            for k in 0..h {
                let jumping = h - k;
                let max_l_jump = k + 1;

                for d in 0..4 {
                    if let Some(prev) = last_op {
                        let (p_dr, p_dc) = DIR[prev.d];
                        let prev_target_p =
                            ((prev.r as isize + p_dr * prev.l as isize) as usize) * grid.n
                                + (prev.c as isize + p_dc * prev.l as isize) as usize;
                        if prev_target_p == p && d == OPPOSITE_DIR[prev.d] && prev.l <= max_l_jump {
                            continue;
                        }
                    }

                    let max_reach = grid.max_reach[p][d];
                    let limit_l = max_l_jump.min(max_reach);

                    for l in 1..=limit_l {
                        let (dr, dc) = DIR[d];
                        let nr = (r as isize + dr * l as isize) as usize;
                        let nc = (c as isize + dc * l as isize) as usize;
                        let next_p = nr * grid.n + nc;

                        let g = board.lens[next_p] as usize;
                        if g + jumping <= 8 {
                            moves.push(Op { r, c, k, d, l });
                        }
                    }
                }
            }

            word &= word - 1;
        }
    }
}

// -------------------------------------------------------------
// Chokudai Search の探索ノード
// -------------------------------------------------------------
#[allow(dead_code)]
struct Node {
    board: BitBoard,
    parent: Option<usize>,
    op: Option<Op>,
    depth: usize,
    static_score: i64,
    final_score: i64,
    slimes_left: usize,
    hash: u64,
}

// -------------------------------------------------------------
// プレフィルタ付きビーム挿入
// -------------------------------------------------------------
#[inline(always)]
fn insert_into_beam(beam_d: &mut Vec<usize>, new_id: usize, arena: &[Node], width: usize) {
    let new_score = arena[new_id].final_score;

    if beam_d.len() >= width {
        let worst_id = *beam_d.last().unwrap();
        if new_score <= arena[worst_id].final_score {
            return; // 最下位スコア以下のため挿入不要
        }
    }

    let pos = match beam_d.binary_search_by(|&id| new_score.cmp(&arena[id].final_score)) {
        Ok(idx) => idx,
        Err(idx) => idx,
    };
    beam_d.insert(pos, new_id);
    if beam_d.len() > width {
        beam_d.pop();
    }
}

// -------------------------------------------------------------
// 操作適用（シミュレータ）
// -------------------------------------------------------------
#[inline(always)]
fn apply_op(board: &mut BitBoard, op: Op, grid: &Grid) -> Option<usize> {
    let p = op.r * grid.n + op.c;
    let h = board.lens[p] as usize;
    if op.k >= h {
        return None;
    }
    let jumping = h - op.k;
    let max_l = op.k + 1;
    if op.l == 0 || op.l > max_l || op.l > grid.max_reach[p][op.d] {
        return None;
    }
    let (dr, dc) = DIR[op.d];
    let nr = (op.r as isize + dr * op.l as isize) as usize;
    let nc = (op.c as isize + dc * op.l as isize) as usize;
    let next_p = nr * grid.n + nc;
    if board.lens[next_p] as usize + jumping > 8 {
        return None;
    }

    board.transfer_and_reverse(p, next_p, op.k);
    let mut cleared = 0;
    if let Some(col) = grid.nest_color_at[p] {
        cleared += board.clean_nest(p, col);
    }
    if let Some(col) = grid.nest_color_at[next_p] {
        cleared += board.clean_nest(next_p, col);
    }

    if board.lens[p] == 0 {
        board.clear_occupied(p);
    }
    if board.lens[next_p] > 0 {
        board.set_occupied(next_p);
    } else {
        board.clear_occupied(next_p);
    }

    Some(cleared)
}

#[inline(always)]
fn simulate_all(mut board: BitBoard, ops: &[Op], grid: &Grid) -> Option<usize> {
    let mut cleared_total = 0;
    for &op in ops {
        if let Some(c) = apply_op(&mut board, op, grid) {
            cleared_total += c;
        } else {
            return None;
        }
    }
    Some(cleared_total)
}

// C_01: exact interval validation and bounded compact-state search.
#[derive(Default, Clone, Copy)]
struct PatchStats {
    trials: usize, accepted: usize, saved_ops: usize, replay_steps: usize,
    rejoin_accepts: usize, compact_nodes: usize, budget_stops: usize,
}
thread_local! {
    static PATCH_STATS: std::cell::RefCell<PatchStats> = std::cell::RefCell::new(PatchStats::default());
}
fn stat(f: impl FnOnce(&mut PatchStats)) { PATCH_STATS.with(|s| f(&mut s.borrow_mut())); }
fn stats_snapshot() -> PatchStats { PATCH_STATS.with(|s| *s.borrow()) }
pub fn post_mode() -> String {
    std::env::args().find_map(|s| s.strip_prefix("--post=").map(str::to_owned))
        .unwrap_or_else(|| "local".into())
}
pub fn print_patch_stats() {
    PATCH_STATS.with(|s| {
        let s = s.borrow();
        eprintln!("PATCH: trials={} accepted={} saved_ops={} replay_steps={} rejoin_accepts={} compact_nodes={} budget_stops={}",
            s.trials, s.accepted, s.saved_ops, s.replay_steps, s.rejoin_accepts, s.compact_nodes, s.budget_stops);
    });
}
fn phase_log(name: &str, begin: Instant, before: usize, after: usize) {
    eprintln!("PHASE: name={} elapsed_ms={:.3} before={} after={} saved_ops={}",
        name, begin.elapsed().as_secs_f64()*1000.0, before, after, before.saturating_sub(after));
}
fn phase_stats(name: &str,before: PatchStats) {
    let after=stats_snapshot();
    eprintln!("PHASE_STATS: name={} trials={} accepted={} saved_ops={} replay_steps={} rejoin_accepts={} compact_nodes={} budget_stops={}",
        name,after.trials-before.trials,after.accepted-before.accepted,after.saved_ops-before.saved_ops,
        after.replay_steps-before.replay_steps,after.rejoin_accepts-before.rejoin_accepts,
        after.compact_nodes-before.compact_nodes,after.budget_stops-before.budget_stops);
}
fn endpoint(op: Op, grid: &Grid) -> (usize, usize) {
    let (dr, dc) = DIR[op.d];
    (op.r * grid.n + op.c,
     (op.r as isize + dr * op.l as isize) as usize * grid.n
        + (op.c as isize + dc * op.l as isize) as usize)
}
fn deadline_at(start: Instant, ms: u128) -> Instant {
    start + std::time::Duration::from_millis(ms as u64)
}
struct TraceCache {
    boards: Vec<BitBoard>, cleared: Vec<usize>, endpoints: Vec<(usize, usize)>,
}
impl TraceCache {
    fn build(initial: &BitBoard, ops: &[Op], grid: &Grid, deadline: Instant) -> Option<Self> {
        let mut trace = Self { boards: vec![*initial], cleared: vec![0], endpoints: Vec::with_capacity(ops.len()) };
        trace.rebuild_from(0, ops, grid, deadline).then_some(trace)
    }
    fn rebuild_from(&mut self, a: usize, ops: &[Op], grid: &Grid, deadline: Instant) -> bool {
        self.boards.truncate(a+1); self.cleared.truncate(a+1); self.endpoints.truncate(a);
        let mut board = self.boards[a]; let mut cleared = self.cleared[a];
        for (t, &op) in ops.iter().enumerate().skip(a) {
            if t % 32 == 0 && Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return false; }
            let Some(c) = apply_op(&mut board, op, grid) else { return false; };
            cleared += c;
            self.boards.push(board); self.cleared.push(cleared); self.endpoints.push(endpoint(op, grid));
        }
        Instant::now() < deadline
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckResult { Accept, Reject, BudgetExpired }
#[inline]
fn same_cell(a: &BitBoard, b: &BitBoard, p: usize) -> bool {
    a.lens[p] == b.lens[p] && a.colors[p] == b.colors[p]
}
fn validate_patch(ops: &[Op], trace: &TraceCache, a: usize, b: usize,
    replacement: &[Op], grid: &Grid, target: usize, deadline: Instant) -> CheckResult {
    stat(|s| s.trials += 1);
    if Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
    let mut board = trace.boards[a]; let mut cleared = trace.cleared[a];
    let mut touched = [0u64; 7];
    for &(p, q) in &trace.endpoints[a..b] {
        touched[p/64] |= 1 << (p%64); touched[q/64] |= 1 << (q%64);
    }
    for (t, &op) in replacement.iter().enumerate() {
        if t % 32 == 0 && Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
        let Some(c) = apply_op(&mut board, op, grid) else { return CheckResult::Reject; };
        cleared += c;
        let (p,q) = endpoint(op, grid);
        touched[p/64] |= 1 << (p%64); touched[q/64] |= 1 << (q%64);
    }
    let mut diff = [0u64; 7]; let mut count = 0usize;
    for w in 0..7 {
        let mut bits = touched[w];
        while bits != 0 {
            let p = w*64 + bits.trailing_zeros() as usize; bits &= bits-1;
            if !same_cell(&board, &trace.boards[b], p) { diff[w] |= 1 << (p%64); count += 1; }
        }
    }
    let total = *trace.cleared.last().unwrap(); let mut replay_steps = 0;
    for t in b..=ops.len() {
        if count == 0 {
            if Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
            let accepted = cleared + total - trace.cleared[t] >= target;
            stat(|s| { s.replay_steps += replay_steps; if accepted { s.rejoin_accepts += 1; } });
            return if accepted { CheckResult::Accept } else { CheckResult::Reject };
        }
        if t == ops.len() { break; }
        if replay_steps % 32 == 0 && Instant::now() >= deadline {
            stat(|s| { s.replay_steps += replay_steps; s.budget_stops += 1; }); return CheckResult::BudgetExpired;
        }
        let Some(c) = apply_op(&mut board, ops[t], grid) else {
            stat(|s| s.replay_steps += replay_steps); return CheckResult::Reject;
        };
        cleared += c; replay_steps += 1;
        let (p,q) = trace.endpoints[t];
        for x in [p,q] {
            let mask = 1u64 << (x%64); let was = diff[x/64] & mask != 0;
            let now = !same_cell(&board, &trace.boards[t+1], x);
            if was != now { if now { count += 1; diff[x/64] |= mask; } else { count -= 1; diff[x/64] &= !mask; } }
        }
    }
    stat(|s| s.replay_steps += replay_steps);
    if Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
    if cleared >= target { CheckResult::Accept } else { CheckResult::Reject }
}
fn record_patch(old: usize, new: usize) { stat(|s| { s.accepted += 1; s.saved_ops += old-new; }); }
fn ray(p: usize, q: usize, grid: &Grid) -> Option<(usize, usize)> {
    if p == q || !grid.is_floor[p] || !grid.is_floor[q] { return None; }
    let (pr,pc,qr,qc) = (p/grid.n,p%grid.n,q/grid.n,q%grid.n);
    let (d,l) = if pr == qr { if pc < qc { (3,qc-pc) } else { (2,pc-qc) } }
        else if pc == qc { if pr < qr { (1,qr-pr) } else { (0,pr-qr) } } else { return None; };
    (l <= grid.max_reach[p][d]).then_some((d,l))
}
fn bridges(p: usize, q: usize, grid: &Grid) -> Vec<usize> {
    let (pr,pc,qr,qc) = (p/grid.n,p%grid.n,q/grid.n,q%grid.n);
    let xs: Vec<usize> = if pr == qr { ((pc.min(qc)+1)..pc.max(qc)).map(|c| pr*grid.n+c).collect() }
        else if pc == qc { ((pr.min(qr)+1)..pr.max(qr)).map(|r| r*grid.n+pc).collect() }
        else { vec![pr*grid.n+qc, qr*grid.n+pc] };
    xs.into_iter().filter(|&x| ray(p,x,grid).is_some() && ray(x,q,grid).is_some()).collect()
}
pub fn complete_2hop_jumps(mut ops: Vec<Op>, initial: &BitBoard, grid: &Grid,
    target: usize, deadline: Instant) -> Vec<Op> {
    let begin = Instant::now(); let before = ops.len(); let stats_before=stats_snapshot();
    let Some(mut trace) = TraceCache::build(initial,&ops,grid,deadline) else { return ops; };
    let mut a = 0;
    'search: while a+3 <= ops.len() && Instant::now() < deadline {
        let p = trace.endpoints[a].0;
        for len in 3..=8 {
            let b = a+len; if b > ops.len() { break; }
            let q = trace.endpoints[b-1].1;
            let mut fallback = Vec::new();
            for x in bridges(p,q,grid) {
                let (d1,l1) = ray(p,x,grid).unwrap(); let (d2,l2) = ray(x,q,grid).unwrap();
                for k1 in (l1-1)..trace.boards[a].lens[p] as usize {
                    if Instant::now() >= deadline { break 'search; }
                    let op1 = Op { r:p/grid.n,c:p%grid.n,k:k1,d:d1,l:l1 };
                    let mut after1 = trace.boards[a];
                    if apply_op(&mut after1,op1,grid).is_none() { continue; }
                    for k2 in (l2-1)..after1.lens[x] as usize {
                        let op2 = Op { r:x/grid.n,c:x%grid.n,k:k2,d:d2,l:l2 };
                        let mut after2 = after1;
                        if apply_op(&mut after2,op2,grid).is_none() { continue; }
                        let patch = [op1,op2];
                        if after2 != trace.boards[b] { fallback.push(patch); continue; }
                        if validate_patch(&ops,&trace,a,b,&patch,grid,target,deadline) == CheckResult::Accept {
                            ops.splice(a..b,patch); record_patch(len,2);
                            if !trace.rebuild_from(a,&ops,grid,deadline) { break 'search; }
                            a = a.saturating_sub(1); continue 'search;
                        }
                    }
                }
            }
            for patch in fallback {
                match validate_patch(&ops,&trace,a,b,&patch,grid,target,deadline) {
                    CheckResult::Accept => {
                        ops.splice(a..b,patch); record_patch(len,2);
                        if !trace.rebuild_from(a,&ops,grid,deadline) { break 'search; }
                        a = a.saturating_sub(1); continue 'search;
                    }
                    CheckResult::BudgetExpired => break 'search,
                    CheckResult::Reject => {}
                }
            }
        }
        a += 1;
    }
    phase_log("bridge",begin,before,ops.len()); phase_stats("bridge",stats_before); ops
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct LocalState { cells: [u64;12] }
fn pack_cell(board: &BitBoard,p: usize) -> u64 { board.colors[p] as u64 | ((board.lens[p] as u64)<<32) }
fn local_state(board: &BitBoard,region: &[usize]) -> LocalState {
    let mut s = LocalState { cells:[0;12] };
    for (i,&p) in region.iter().enumerate() { s.cells[i] = pack_cell(board,p); } s
}
#[inline]
fn local_clean(cell: &mut u64,nest: Option<usize>) {
    let Some(c) = nest else { return; };
    let mut h = (*cell>>32) as usize; let mut bits = *cell as u32;
    while h>0 && ((bits>>((h-1)*4))&15) as usize == c {
        h -= 1; bits &= if h==0 {0} else {(1u32<<(h*4))-1};
    }
    *cell = bits as u64 | ((h as u64)<<32);
}
fn local_move(s: LocalState,i: usize,j: usize,k: usize,l: usize,nests: &[Option<usize>;12]) -> Option<LocalState> {
    let h = (s.cells[i]>>32) as usize; let g = (s.cells[j]>>32) as usize;
    if k>=h || l==0 || l>k+1 || g+h-k>8 { return None; }
    let mut next = s; let mut bits = (s.cells[i] as u32)>>(k*4); let mut rev=0u32;
    for _ in k..h { rev = (rev<<4)|(bits&15); bits >>= 4; }
    let mask = if k==0 {0} else {(1u32<<(k*4))-1};
    next.cells[i] = ((s.cells[i] as u32 & mask) as u64)|((k as u64)<<32);
    next.cells[j] = ((s.cells[j] as u32 | (rev<<(g*4))) as u64)|(((g+h-k) as u64)<<32);
    local_clean(&mut next.cells[i],nests[i]); local_clean(&mut next.cells[j],nests[j]); Some(next)
}
fn color_counts(s: &LocalState) -> [u8;16] {
    let mut counts=[0;16];
    for &cell in &s.cells {
        let mut bits=cell as u32;
        for _ in 0..(cell>>32) { counts[(bits&15) as usize]+=1; bits>>=4; }
    } counts
}
struct LocalSearch<'a> {
    region: &'a [usize], n: usize, nests: [Option<usize>;12], rays: [[Option<(usize,usize)>;12];12],
    goal: LocalState, required: [u8;16], seen: FastHashMap<LocalState,usize>,
    deadline: Instant, nodes: usize, stopped: bool,
}
impl LocalSearch<'_> {
    fn dfs(&mut self,s: LocalState,depth: usize,limit: usize,path: &mut Vec<Op>) -> bool {
        if self.nodes >= 2000 || (self.nodes%32==0 && Instant::now()>=self.deadline) {
            self.stopped=true; return false;
        }
        self.nodes+=1;
        if s==self.goal { return true; }
        let diff=s.cells.iter().zip(self.goal.cells.iter()).filter(|(a,b)|a!=b).count();
        if diff>2*(limit-depth) || depth==limit { return false; }
        let counts=color_counts(&s);
        if (0..16).any(|c|counts[c]<self.required[c]) { return false; }
        if self.seen.get(&s).is_some_and(|&d|d<=depth) { return false; }
        self.seen.insert(s,depth);
        let mut moves=Vec::new(); let mut generated=0;
        for i in 0..self.region.len() {
            let h=(s.cells[i]>>32) as usize;
            for j in 0..self.region.len() {
                let Some((d,l))=self.rays[i][j] else {continue;};
                for k in (l-1)..h {
                    generated+=1;
                    if generated%32==0 && Instant::now()>=self.deadline { self.stopped=true; return false; }
                    let Some(next)=local_move(s,i,j,k,l,&self.nests) else {continue;};
                    if next==s {continue;}
                    let nd=next.cells.iter().zip(self.goal.cells.iter()).filter(|(a,b)|a!=b).count();
                    let p=self.region[i];
                    moves.push((nd,next,Op{r:p/self.n,c:p%self.n,k,d,l}));
                }
            }
        }
        moves.sort_by_key(|x|x.0);
        for (_,next,op) in moves {
            path.push(op);
            if self.dfs(next,depth+1,limit,path) { return true; }
            path.pop(); if self.stopped { return false; }
        }
        false
    }
}
fn compact_window(start: &BitBoard,goal: &BitBoard,region: &[usize],original_len: usize,grid: &Grid,deadline: Instant) -> Option<Vec<Op>> {
    let mut nests=[None;12]; let mut rays=[[None;12];12];
    for (i,&p) in region.iter().enumerate() {
        nests[i]=grid.nest_color_at[p];
        for (j,&q) in region.iter().enumerate() { rays[i][j]=ray(p,q,grid); }
    }
    let initial=local_state(start,region); let goal=local_state(goal,region);
    let mut search=LocalSearch{region,n:grid.n,nests,rays,goal,required:color_counts(&goal),seen:FastHashMap::default(),
        deadline,nodes:0,stopped:false};
    let mut answer=None;
    for limit in 0..=4.min(original_len-1) {
        search.seen.clear(); let mut path=Vec::new();
        if search.dfs(initial,0,limit,&mut path) {
            answer=Some(path); break;
        }
        if search.stopped {break;}
    }
    stat(|s| {s.compact_nodes+=search.nodes; if search.stopped {s.budget_stops+=1;} }); answer
}
fn closure_region(endpoints: &[(usize,usize)],a: usize,b: usize) -> (Vec<usize>,Vec<usize>) {
    let (p,q)=endpoints[a]; let mut region=vec![p,q];
    loop {
        let old=region.len();
        for &(p,q) in &endpoints[a..b] {
            if region.contains(&p)||region.contains(&q) {
                if !region.contains(&p) {region.push(p);}
                if !region.contains(&q) {region.push(q);}
            }
        }
        if old==region.len() {break;}
    }
    let chosen=(a..b).filter(|&t|region.contains(&endpoints[t].0)||region.contains(&endpoints[t].1)).collect();
    (region,chosen)
}
fn add_auxiliary(region: &mut Vec<usize>,start: &BitBoard,endpoints: &[(usize,usize)],grid: &Grid) {
    let mandatory=region.len(); let mut xs=Vec::new();
    for i in 0..mandatory { for j in (i+1)..mandatory {
        for x in bridges(region[i],region[j],grid) {
            if start.lens[x]==0 {xs.push(x);}
        }
    }}
    for &p in &region[..mandatory] {
        for d in 0..4 { if grid.max_reach[p][d]>0 {
            let (dr,dc)=DIR[d]; let x=(p/grid.n) as isize*grid.n as isize + (p%grid.n) as isize + dr*grid.n as isize+dc;
            if start.lens[x as usize]>0 {xs.push(x as usize);}
        }}
    }
    for x in xs {
        if region.len()>=12.min(mandatory+2) {break;}
        if !region.contains(&x) && !endpoints.iter().any(|&(p,q)|p==x||q==x) {region.push(x);}
    }
}
pub fn compact_optimize(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,deadline: Instant,independent: bool) -> Vec<Op> {
    let begin=Instant::now(); let before=ops.len(); let stats_before=stats_snapshot();
    let Some(mut trace)=TraceCache::build(initial,&ops,grid,deadline) else {return ops;};
    let mut a=0;
    'search: while a+3<=ops.len() && Instant::now()<deadline {
        let mut windows: Vec<(usize,bool)>=(3..=6).filter(|&l|a+l<=ops.len()).map(|l|(a+l,false)).collect();
        if independent {
            let mut next_counts=vec![3,4,6];
            for b in (a+3)..=(a+24).min(ops.len()) {
                let (region,chosen)=closure_region(&trace.endpoints,a,b);
                if region.len()>12 || chosen.len()>6 {break;}
                if next_counts.contains(&chosen.len()) {
                    if chosen.len()<b-a {windows.push((b,true));}
                    next_counts.retain(|&c|c!=chosen.len());
                }
            }
        }
        for (b,filtered) in windows {
            if Instant::now()>=deadline {break 'search;}
            let (mut region,chosen)=if filtered {closure_region(&trace.endpoints,a,b)} else {
                let mut cells=Vec::new();
                for &(p,q) in &trace.endpoints[a..b] {for x in [p,q] {if !cells.contains(&x) {cells.push(x);}}}
                (cells,(a..b).collect::<Vec<_>>())
            };
            if region.len()>12 || !(3..=6).contains(&chosen.len()) {continue;}
            add_auxiliary(&mut region,&trace.boards[a],&trace.endpoints[a..b],grid);
            let window_deadline=deadline.min(Instant::now()+std::time::Duration::from_millis(1));
            let Some(mut patch)=compact_window(&trace.boards[a],&trace.boards[b],&region,chosen.len(),grid,window_deadline) else {continue;};
            for t in a..b {if !chosen.contains(&t) {patch.push(ops[t]);}}
            // Mandatory full interval equality check, including preserved independent moves.
            let mut check=trace.boards[a]; let mut legal=true;
            for &op in &patch {if apply_op(&mut check,op,grid).is_none() {legal=false;break;}}
            if legal && check==trace.boards[b] && Instant::now()<deadline {
                record_patch(b-a,patch.len()); ops.splice(a..b,patch);
                if !trace.rebuild_from(a,&ops,grid,deadline) {break 'search;}
                a=a.saturating_sub(1); continue 'search;
            }
        }
        a+=1;
    }
    phase_log("compact",begin,before,ops.len()); phase_stats("compact",stats_before); ops
}

pub fn finish_new_pipeline(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,start: Instant,mode: &str) -> Vec<Op> {
    let bridge_deadline=deadline_at(start,1680).min(Instant::now()+std::time::Duration::from_millis(30));
    if Instant::now()<bridge_deadline { ops=complete_2hop_jumps(ops,initial,grid,grid.initial_slimes,bridge_deadline); }
    if mode=="local"||mode=="local-contiguous" {
        let local_deadline=deadline_at(start,1730).min(Instant::now()+std::time::Duration::from_millis(50));
        if Instant::now()<local_deadline {ops=compact_optimize(ops,initial,grid,local_deadline,mode=="local");}
    } else if start.elapsed().as_millis()+150<1730 {
        ops=lns_optimize(ops,initial,grid,start,1730);
    }
    let begin=Instant::now(); let before=ops.len();
    ops=prune_ops(ops,initial,grid,grid.initial_slimes,start,1750);
    ops=prune_cycle_and_uturn_ops(ops,initial,grid,grid.initial_slimes,start,1750);
    phase_log("final",begin,before,ops.len()); ops
}

pub fn valid_complete(initial: &BitBoard,ops: &[Op],grid: &Grid) -> bool { simulate_all(*initial,ops,grid)==Some(grid.initial_slimes) }
#[cfg(test)]
mod c01_tests {
    use super::*;
    fn fixture(n:usize) -> (Grid,BitBoard) {
        let mut raw=vec![vec!['.';n];n]; raw[0][2]='A'; raw[n-1][n-1]='B';
        Grid::new(n,2,&raw)
    }
    fn put(board:&mut BitBoard,p:usize,colors:&[usize]) {
        board.colors[p]=0; board.lens[p]=colors.len() as u8;
        for (i,&c) in colors.iter().enumerate() {board.colors[p]|=(c as u32)<<(i*4);}
        if colors.is_empty() {board.clear_occupied(p);} else {board.set_occupied(p);}
    }
    fn long_route() -> Vec<Op> {
        vec![Op{r:0,c:0,k:0,d:1,l:1},Op{r:1,c:0,k:0,d:3,l:1},
            Op{r:1,c:1,k:0,d:3,l:1},Op{r:1,c:2,k:0,d:0,l:1}]
    }
    fn budget() -> Instant {Instant::now()+std::time::Duration::from_secs(5)}
    #[test]
    fn capacity_and_both_nests() {
        let (mut grid,mut board)=fixture(5);
        put(&mut board,0,&[0,0,1]); grid.nest_color_at[0]=Some(0); grid.nest_color_at[3]=Some(1);
        assert_eq!(apply_op(&mut board,Op{r:0,c:0,k:2,d:3,l:3},&grid),Some(3));
        assert!(board.lens.iter().all(|&h|h==0));
        put(&mut board,0,&[1,0]); put(&mut board,1,&[1,1,1,1,1,1]);
        assert_eq!(apply_op(&mut board,Op{r:0,c:0,k:0,d:3,l:1},&grid),Some(0));
        assert_eq!(board.lens[1],8);
        put(&mut board,0,&[1]); grid.nest_color_at[1]=Some(1);
        let original=board;
        assert_eq!(apply_op(&mut board,Op{r:0,c:0,k:0,d:3,l:1},&grid),None);
        assert!(board==original);
    }
    #[test]
    fn compact_transition_matches_reference() {
        let (grid,mut board)=fixture(5); let mut rng=887766554433u64;
        let region=[0,1,2,3,4,5,6,7,8,9,10,11];
        let mut nests=[None;12]; for i in 0..12 {nests[i]=grid.nest_color_at[region[i]];}
        let mut samples=0;
        for _ in 0..200 {
            for &p in &region {
                rng^=rng<<13; rng^=rng>>7; rng^=rng<<17;
                let h=(rng%9) as usize;
                let colors:Vec<usize>=(0..h).map(|i|((rng>>(i*4))&1) as usize).collect();
                put(&mut board,p,&colors);
            }
            let s=local_state(&board,&region);
            for i in 0..12 {for j in 0..12 {
                let Some((d,l))=ray(region[i],region[j],&grid) else {continue;};
                for k in 0..=8 {
                    let mut full=board;
                    let result=apply_op(&mut full,Op{r:region[i]/5,c:region[i]%5,k,d,l},&grid);
                    let compact=local_move(s,i,j,k,l,&nests);
                    assert_eq!(result.is_some(),compact.is_some());
                    if let Some(next)=compact {assert_eq!(next,local_state(&full,&region));}
                    samples+=1;
                }
            }}
        }
        assert!(samples>10000);
        put(&mut board,0,&[0;8]);
        assert_ne!(pack_cell(&board,0),0);
    }
    #[test]
    fn differential_validation_matches_full_replay() {
        let (grid,mut initial)=fixture(5); put(&mut initial,0,&[0]); put(&mut initial,20,&[1;8]);
        let mut board=initial; let mut ops=long_route();
        for &op in &ops {apply_op(&mut board,op,&grid).unwrap();}
        let mut rng=9988776655u64;
        for _ in 0..45 {
            let mut moves=Vec::new(); generate_valid_moves_into(&board,None,&grid,&mut moves);
            if moves.is_empty() {break;}
            rng^=rng<<13; rng^=rng>>7; rng^=rng<<17;
            let op=moves[(rng as usize)%moves.len()]; apply_op(&mut board,op,&grid).unwrap(); ops.push(op);
        }
        let trace=TraceCache::build(&initial,&ops,&grid,budget()).unwrap();
        let replacements=vec![vec![],vec![Op{r:0,c:0,k:0,d:3,l:1},Op{r:0,c:1,k:0,d:3,l:1}],
            vec![Op{r:0,c:0,k:7,d:3,l:1}]];
        let mut checks=0;
        for a in 0..ops.len() {for b in a+1..=(a+8).min(ops.len()) {
            let unchanged=ops[a..b].to_vec();
            for patch in replacements.iter().chain(std::iter::once(&unchanged)) {
                for target in [0,1,grid.initial_slimes+9] {
                    let mut full=ops.clone();full.splice(a..b,patch.iter().copied());
                    let expected=simulate_all(initial,&full,&grid).is_some_and(|c|c>=target);
                    assert_eq!(validate_patch(&ops,&trace,a,b,patch,&grid,target,budget())==CheckResult::Accept,expected);
                    checks+=1;
                }
            }
        }}
        assert!(checks>1000);
        let original=ops.clone();
        assert_eq!(validate_patch(&ops,&trace,0,4,&[],&grid,1,Instant::now()),CheckResult::BudgetExpired);
        assert_eq!(ops,original);
        assert_eq!(complete_2hop_jumps(ops.clone(),&initial,&grid,1,Instant::now()),ops);
        assert_eq!(compact_optimize(ops.clone(),&initial,&grid,Instant::now(),true),ops);
    }
    #[test]
    fn empty_straight_bridge_and_exact_compact_search() {
        let (grid,mut initial)=fixture(5); put(&mut initial,0,&[0]);
        let ops=long_route();
        let short=complete_2hop_jumps(ops.clone(),&initial,&grid,1,budget());
        assert_eq!(short.len(),2); assert_eq!(simulate_all(initial,&short,&grid),Some(1));
        let mut goal=initial;for &op in &ops {apply_op(&mut goal,op,&grid).unwrap();}
        let region=[0,5,6,7,2,1];
        let compact=compact_window(&initial,&goal,&region,4,&grid,budget()).unwrap();
        assert_eq!(compact.len(),2);
        let mut end=initial;for op in compact {apply_op(&mut end,op,&grid).unwrap();} assert!(end==goal);
    }
    #[test]
    fn dependency_closure_and_preserved_independent_move() {
        let pairs=[(0,1),(4,5),(1,4),(2,3)];
        let (r,chosen)=closure_region(&pairs,0,4);
        assert!(r.contains(&5));assert_eq!(chosen,vec![0,1,2]);
        let (grid,mut initial)=fixture(5);put(&mut initial,0,&[0]);put(&mut initial,20,&[1]);
        let mut ops=long_route();ops.insert(1,Op{r:4,c:0,k:0,d:3,l:1});
        let trace=TraceCache::build(&initial,&ops,&grid,budget()).unwrap();
        let (mut r,chosen)=closure_region(&trace.endpoints,0,5);
        assert_eq!(chosen.len(),4); assert!(!r.contains(&20));
        add_auxiliary(&mut r,&initial,&trace.endpoints,&grid);
        let mut patch=compact_window(&initial,&trace.boards[5],&r,4,&grid,budget()).unwrap();
        patch.push(ops[1]);
        let mut end=initial;for op in patch {apply_op(&mut end,op,&grid).unwrap();}
        assert!(end==trace.boards[5]);
    }
    #[test]
    fn external_tower_can_be_overflown_but_not_landed_on() {
        let (grid,mut initial)=fixture(5);put(&mut initial,0,&[1,0,1]);put(&mut initial,1,&[1;8]);
        let region=[0,2];let mut nests=[None;12];nests[1]=Some(0);
        let s=local_state(&initial,&region);
        let next=local_move(s,0,1,1,2,&nests).unwrap();
        let mut full=initial;assert!(apply_op(&mut full,Op{r:0,c:0,k:1,d:3,l:2},&grid).is_some());
        assert_eq!(next,local_state(&full,&region));assert_eq!(full.lens[1],8);
        let goal=full;
        let answer=compact_window(&initial,&goal,&region,3,&grid,budget()).unwrap();
        assert!(answer.iter().all(|op|region.contains(&endpoint(*op,&grid).1)));
    }
}

// -------------------------------------------------------------
// 過去改変 第2弾: 同一直線上の 2 手連続ジャンプの 1 手マージ（ショートカット）
// -------------------------------------------------------------
pub fn merge_straight_jumps(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
    start_time: Instant,
    deadline_ms: u128,
) -> Vec<Op> {
    let mut improved = true;
    while improved {
        if start_time.elapsed().as_millis() >= deadline_ms {
            break;
        }
        improved = false;
        let n_ops = ops.len();

        let deadline = deadline_at(start_time, deadline_ms);
        let Some(trace) = TraceCache::build(initial_board, &ops, grid, deadline) else { break; };
        let prefix_boards = &trace.boards;
        let mut replacement = Vec::with_capacity(64);

        'outer: for i in 0..n_ops {
            if start_time.elapsed().as_millis() >= deadline_ms {
                break 'outer;
            }
            let op_i = ops[i];
            let (dr, dc) = DIR[op_i.d];
            let dest_r = op_i.r as isize + dr * op_i.l as isize;
            let dest_c = op_i.c as isize + dc * op_i.l as isize;
            let p_i = op_i.r * grid.n + op_i.c;

            // i の直後から探索（最大 60 手先まで）
            let limit_j = (i + 60).min(n_ops);
            for j in (i + 1)..limit_j {
                let op_j = ops[j];
                // 手 j の始点が手 i の着地先で、かつ同じ方向
                if op_j.r as isize == dest_r && op_j.c as isize == dest_c && op_j.d == op_i.d {
                    let total_l = op_i.l + op_j.l;
                    if total_l > grid.max_reach[p_i][op_i.d] {
                        continue;
                    }

                    let h_before_i = prefix_boards[i].lens[p_i] as usize;

                    let mut k_candidates = Vec::new();
                    if total_l <= op_i.k + 1 && op_i.k < h_before_i {
                        k_candidates.push(op_i.k);
                    }
                    if total_l - 1 < h_before_i {
                        k_candidates.push(total_l - 1);
                    }
                    for k_cand in (total_l - 1)..h_before_i {
                        if !k_candidates.contains(&k_cand) {
                            k_candidates.push(k_cand);
                        }
                    }

                    for k_cand in k_candidates {
                        replacement.clear();
                        let mut changed = op_i;
                        changed.l = total_l; changed.k = k_cand;
                        replacement.push(changed);
                        replacement.extend_from_slice(&ops[i+1..j]);
                        match validate_patch(&ops, &trace, i, j+1, &replacement, grid, target_cleared, deadline) {
                            CheckResult::Accept => {
                                record_patch(j+1-i, replacement.len());
                                ops.splice(i..j+1, replacement.iter().copied());
                                improved = true;
                                break 'outer;
                            }
                            CheckResult::BudgetExpired => break 'outer,
                            CheckResult::Reject => {}
                        }
                    }
                }
            }
        }
    }
    ops
}

// -------------------------------------------------------------
// 幾何学的多段ショートカット（Multi-hop Ray Shortcut）
// -------------------------------------------------------------
pub fn shortcut_ray_jumps(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
    start_time: Instant,
    deadline_ms: u128,
) -> Vec<Op> {
    let mut improved = true;
    while improved {
        if start_time.elapsed().as_millis() >= deadline_ms {
            break;
        }
        improved = false;
        let n_ops = ops.len();

        let deadline = deadline_at(start_time, deadline_ms);
        let Some(trace) = TraceCache::build(initial_board, &ops, grid, deadline) else { break; };
        let prefix_boards = &trace.boards;
        let mut replacement = Vec::with_capacity(64);

        'outer: for i in 0..n_ops {
            if start_time.elapsed().as_millis() >= deadline_ms {
                break 'outer;
            }
            let op_i = ops[i];
            let p_i = op_i.r * grid.n + op_i.c;
            let h_before_i = prefix_boards[i].lens[p_i] as usize;

            let limit_j = (i + 40).min(n_ops);
            for j in (i + 1)..limit_j {
                let op_j = ops[j];
                let (dr_j, dc_j) = DIR[op_j.d];
                let dest_r = op_j.r as isize + dr_j * op_j.l as isize;
                let dest_c = op_j.c as isize + dc_j * op_j.l as isize;

                let dr = dest_r - op_i.r as isize;
                let dc = dest_c - op_i.c as isize;

                let direct_dir_opt = if dr == 0 && dc > 0 {
                    Some((3, dc as usize))
                } else if dr == 0 && dc < 0 {
                    Some((2, (-dc) as usize))
                } else if dc == 0 && dr > 0 {
                    Some((1, dr as usize))
                } else if dc == 0 && dr < 0 {
                    Some((0, (-dr) as usize))
                } else {
                    None
                };

                let Some((d_dir, dist_l)) = direct_dir_opt else {
                    continue;
                };

                if dist_l > grid.max_reach[p_i][d_dir] || dist_l == 0 {
                    continue;
                }

                if dist_l > h_before_i {
                    continue;
                }

                let mut k_candidates = Vec::new();
                if dist_l - 1 < h_before_i {
                    k_candidates.push(dist_l - 1);
                }
                for k_cand in dist_l..h_before_i {
                    if !k_candidates.contains(&k_cand) {
                        k_candidates.push(k_cand);
                    }
                }

                for k_cand in k_candidates {
                    replacement.clear();
                    replacement.push(Op { r:op_i.r,c:op_i.c,k:k_cand,d:d_dir,l:dist_l });
                    replacement.extend_from_slice(&ops[i+1..j]);
                    match validate_patch(&ops, &trace, i, j+1, &replacement, grid, target_cleared, deadline) {
                        CheckResult::Accept => {
                            record_patch(j+1-i, replacement.len());
                            ops.splice(i..j+1, replacement.iter().copied());
                            improved = true;
                            break 'outer;
                        }
                        CheckResult::BudgetExpired => break 'outer,
                        CheckResult::Reject => {}
                    }
                }
            }
        }
    }
    ops
}

// -------------------------------------------------------------
// 直交2段ジャンプ（2-Hop Orthogonal Warp）
// -------------------------------------------------------------
pub fn orthogonal_2hop_jumps(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
    start_time: Instant,
    deadline_ms: u128,
) -> Vec<Op> {
    let mut improved = true;
    while improved {
        if start_time.elapsed().as_millis() >= deadline_ms {
            break;
        }
        improved = false;
        let n_ops = ops.len();

        let deadline = deadline_at(start_time, deadline_ms);
        let Some(trace) = TraceCache::build(initial_board, &ops, grid, deadline) else { break; };
        let prefix_boards = &trace.boards;
        let mut replacement = Vec::with_capacity(64);

        'outer: for i in 0..n_ops {
            if start_time.elapsed().as_millis() >= deadline_ms {
                break 'outer;
            }
            let op_i = ops[i];
            let p_i = op_i.r * grid.n + op_i.c;
            let h_before_i = prefix_boards[i].lens[p_i] as usize;

            let limit_j = (i + 30).min(n_ops);
            for j in (i + 2)..limit_j {
                let op_j = ops[j];
                let (dr_j, dc_j) = DIR[op_j.d];
                let dest_r = (op_j.r as isize + dr_j * op_j.l as isize) as usize;
                let dest_c = (op_j.c as isize + dc_j * op_j.l as isize) as usize;

                if op_i.r == dest_r || op_i.c == dest_c {
                    continue;
                }

                let corners = [
                    (op_i.r, dest_c),
                    (dest_r, op_i.c),
                ];

                for &(cr, cc) in &corners {
                    let p_corner = cr * grid.n + cc;
                    if !grid.is_floor[p_corner] {
                        continue;
                    }

                    let dr1 = cr as isize - op_i.r as isize;
                    let dc1 = cc as isize - op_i.c as isize;
                    let (d1, l1) = if dr1 == 0 && dc1 > 0 {
                        (3, dc1 as usize)
                    } else if dr1 == 0 && dc1 < 0 {
                        (2, (-dc1) as usize)
                    } else if dc1 == 0 && dr1 > 0 {
                        (1, dr1 as usize)
                    } else {
                        (0, (-dr1) as usize)
                    };

                    if l1 > grid.max_reach[p_i][d1] || l1 > h_before_i {
                        continue;
                    }

                    let dr2 = dest_r as isize - cr as isize;
                    let dc2 = dest_c as isize - cc as isize;
                    let (d2, l2) = if dr2 == 0 && dc2 > 0 {
                        (3, dc2 as usize)
                    } else if dr2 == 0 && dc2 < 0 {
                        (2, (-dc2) as usize)
                    } else if dc2 == 0 && dr2 > 0 {
                        (1, dr2 as usize)
                    } else {
                        (0, (-dr2) as usize)
                    };

                    if l2 > grid.max_reach[p_corner][d2] {
                        continue;
                    }

                    let k1 = l1 - 1;
                    let flying1 = h_before_i - k1;
                    if flying1 < l2 {
                        continue;
                    }
                    let k2 = l2 - 1;

                    replacement.clear();
                    replacement.push(Op {
                                            r: op_i.r,
                                            c: op_i.c,
                                            k: k1,
                                            d: d1,
                                            l: l1,
                                        });
                    replacement.push(Op {
                                            r: cr,
                                            c: cc,
                                            k: k2,
                                            d: d2,
                                            l: l2,
                                        });
                    match validate_patch(&ops, &trace, i, j+1, &replacement, grid, target_cleared, deadline) {
                        CheckResult::Accept => {
                            record_patch(j+1-i, replacement.len());
                            ops.splice(i..j+1, replacement.iter().copied());
                            improved = true;
                            break 'outer;
                        }
                        CheckResult::BudgetExpired => break 'outer,
                        CheckResult::Reject => {}
                    }
                }
            }
        }
    }
    ops
}

// -------------------------------------------------------------
// 相乗り大ジャンプ（Co-riding Leap Shortcut）
// -------------------------------------------------------------
pub fn coriding_leap_jumps(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
    start_time: Instant,
    deadline_ms: u128,
) -> Vec<Op> {
    let mut improved = true;
    while improved {
        if start_time.elapsed().as_millis() >= deadline_ms {
            break;
        }
        improved = false;
        let n_ops = ops.len();

        let deadline = deadline_at(start_time, deadline_ms);
        let Some(trace) = TraceCache::build(initial_board, &ops, grid, deadline) else { break; };
        let prefix_boards = &trace.boards;
        let mut replacement = Vec::with_capacity(64);

        'outer: for i in 0..n_ops {
            if start_time.elapsed().as_millis() >= deadline_ms {
                break 'outer;
            }
            let op_i = ops[i];
            let p_i = op_i.r * grid.n + op_i.c;
            let h_before_i = prefix_boards[i].lens[p_i] as usize;
            if h_before_i == 0 {
                continue;
            }

            let limit_j = (i + 30).min(n_ops);
            for j in (i + 2)..limit_j {
                let op_j = ops[j];
                let (dr_j, dc_j) = DIR[op_j.d];
                let dest_r_i = op_j.r as isize + dr_j * op_j.l as isize;
                let dest_c_i = op_j.c as isize + dc_j * op_j.l as isize;
                if dest_r_i < 0 || dest_r_i >= grid.n as isize || dest_c_i < 0 || dest_c_i >= grid.n as isize {
                    continue;
                }
                let dest_r = dest_r_i as usize;
                let dest_c = dest_c_i as usize;
                let p_dest = dest_r * grid.n + dest_c;
                if !grid.is_floor[p_dest] {
                    continue;
                }

                let mut candidates = Vec::new();

                if op_i.r == dest_r {
                    let min_c = op_i.c.min(dest_c);
                    let max_c = op_i.c.max(dest_c);
                    for mc in (min_c + 1)..max_c {
                        let p_m = op_i.r * grid.n + mc;
                        if grid.is_floor[p_m] && prefix_boards[i].lens[p_m] > 0 {
                            candidates.push((op_i.r, mc));
                        }
                    }
                } else if op_i.c == dest_c {
                    let min_r = op_i.r.min(dest_r);
                    let max_r = op_i.r.max(dest_r);
                    for mr in (min_r + 1)..max_r {
                        let p_m = mr * grid.n + op_i.c;
                        if grid.is_floor[p_m] && prefix_boards[i].lens[p_m] > 0 {
                            candidates.push((mr, op_i.c));
                        }
                    }
                } else {
                    let corners = [(op_i.r, dest_c), (dest_r, op_i.c)];
                    for &(cr, cc) in &corners {
                        let p_c = cr * grid.n + cc;
                        if grid.is_floor[p_c] && prefix_boards[i].lens[p_c] > 0 {
                            candidates.push((cr, cc));
                        }
                    }
                }

                for &(mr, mc) in &candidates {
                    let p_mid = mr * grid.n + mc;
                    let h_mid = prefix_boards[i].lens[p_mid] as usize;

                    let dr1 = mr as isize - op_i.r as isize;
                    let dc1 = mc as isize - op_i.c as isize;
                    let (d1, l1) = if dr1 == 0 && dc1 > 0 {
                        (3, dc1 as usize)
                    } else if dr1 == 0 && dc1 < 0 {
                        (2, (-dc1) as usize)
                    } else if dc1 == 0 && dr1 > 0 {
                        (1, dr1 as usize)
                    } else if dc1 == 0 && dr1 < 0 {
                        (0, (-dr1) as usize)
                    } else {
                        continue;
                    };

                    if l1 > grid.max_reach[p_i][d1] || l1 > h_before_i {
                        continue;
                    }

                    let dr2 = dest_r as isize - mr as isize;
                    let dc2 = dest_c as isize - mc as isize;
                    let (d2, l2) = if dr2 == 0 && dc2 > 0 {
                        (3, dc2 as usize)
                    } else if dr2 == 0 && dc2 < 0 {
                        (2, (-dc2) as usize)
                    } else if dc2 == 0 && dr2 > 0 {
                        (1, dr2 as usize)
                    } else if dc2 == 0 && dr2 < 0 {
                        (0, (-dr2) as usize)
                    } else {
                        continue;
                    };

                    if l2 > grid.max_reach[p_mid][d2] {
                        continue;
                    }

                    let k1 = l1 - 1;
                    let flying1 = h_before_i - k1;
                    let total_h_mid = flying1 + h_mid;
                    if total_h_mid < l2 {
                        continue;
                    }
                    let k2 = l2 - 1;

                    replacement.clear();
                    replacement.push(Op {
                                            r: op_i.r,
                                            c: op_i.c,
                                            k: k1,
                                            d: d1,
                                            l: l1,
                                        });
                    replacement.push(Op {
                                            r: mr,
                                            c: mc,
                                            k: k2,
                                            d: d2,
                                            l: l2,
                                        });
                    match validate_patch(&ops, &trace, i, j+1, &replacement, grid, target_cleared, deadline) {
                        CheckResult::Accept => {
                            record_patch(j+1-i, replacement.len());
                            ops.splice(i..j+1, replacement.iter().copied());
                            improved = true;
                            break 'outer;
                        }
                        CheckResult::BudgetExpired => break 'outer,
                        CheckResult::Reject => {}
                    }
                }
            }
        }
    }
    ops
}

// -------------------------------------------------------------
// 直交3段クランクジャンプ（3-Hop Crank Warp / Z-Warp）
// -------------------------------------------------------------
pub fn crank_3hop_jumps(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
    start_time: Instant,
    deadline_ms: u128,
) -> Vec<Op> {
    let mut improved = true;
    while improved {
        if start_time.elapsed().as_millis() >= deadline_ms {
            break;
        }
        improved = false;
        let n_ops = ops.len();

        let deadline = deadline_at(start_time, deadline_ms);
        let Some(trace) = TraceCache::build(initial_board, &ops, grid, deadline) else { break; };
        let prefix_boards = &trace.boards;
        let mut replacement = Vec::with_capacity(64);

        'outer: for i in 0..n_ops {
            if start_time.elapsed().as_millis() >= deadline_ms {
                break 'outer;
            }
            let op_i = ops[i];
            let p_i = op_i.r * grid.n + op_i.c;
            let h_before_i = prefix_boards[i].lens[p_i] as usize;
            if h_before_i < 2 {
                continue;
            }

            let limit_j = (i + 30).min(n_ops);
            for j in (i + 3)..limit_j {
                let op_j = ops[j];
                let (dr_j, dc_j) = DIR[op_j.d];
                let dest_r_i = op_j.r as isize + dr_j * op_j.l as isize;
                let dest_c_i = op_j.c as isize + dc_j * op_j.l as isize;
                if dest_r_i < 0 || dest_r_i >= grid.n as isize || dest_c_i < 0 || dest_c_i >= grid.n as isize {
                    continue;
                }
                let dest_r = dest_r_i as usize;
                let dest_c = dest_c_i as usize;
                if op_i.r == dest_r || op_i.c == dest_c {
                    continue;
                }

                let p_dest = dest_r * grid.n + dest_c;
                if !grid.is_floor[p_dest] {
                    continue;
                }

                let min_r = op_i.r.min(dest_r);
                let max_r = op_i.r.max(dest_r);
                let min_c = op_i.c.min(dest_c);
                let max_c = op_i.c.max(dest_c);

                let mut path_candidates = Vec::new();

                // パターン 1: 縦 -> 横 -> 縦 (中間行 r_mid を経由)
                for r_mid in (min_r + 1)..max_r {
                    path_candidates.push(((r_mid, op_i.c), (r_mid, dest_c)));
                }

                // パターン 2: 横 -> 縦 -> 横 (中間列 c_mid を経由)
                for c_mid in (min_c + 1)..max_c {
                    path_candidates.push(((op_i.r, c_mid), (dest_r, c_mid)));
                }

                for &((m1_r, m1_c), (m2_r, m2_c)) in &path_candidates {
                    let p_m1 = m1_r * grid.n + m1_c;
                    let p_m2 = m2_r * grid.n + m2_c;
                    if !grid.is_floor[p_m1] || !grid.is_floor[p_m2] {
                        continue;
                    }

                    // Hop 1: (op_i.r, op_i.c) -> (m1_r, m1_c)
                    let dr1 = m1_r as isize - op_i.r as isize;
                    let dc1 = m1_c as isize - op_i.c as isize;
                    let (d1, l1) = if dr1 == 0 && dc1 > 0 { (3, dc1 as usize) }
                    else if dr1 == 0 && dc1 < 0 { (2, (-dc1) as usize) }
                    else if dc1 == 0 && dr1 > 0 { (1, dr1 as usize) }
                    else if dc1 == 0 && dr1 < 0 { (0, (-dr1) as usize) }
                    else { continue; };

                    if l1 > grid.max_reach[p_i][d1] || l1 > h_before_i {
                        continue;
                    }

                    // Hop 2: (m1_r, m1_c) -> (m2_r, m2_c)
                    let dr2 = m2_r as isize - m1_r as isize;
                    let dc2 = m2_c as isize - m1_c as isize;
                    let (d2, l2) = if dr2 == 0 && dc2 > 0 { (3, dc2 as usize) }
                    else if dr2 == 0 && dc2 < 0 { (2, (-dc2) as usize) }
                    else if dc2 == 0 && dr2 > 0 { (1, dr2 as usize) }
                    else if dc2 == 0 && dr2 < 0 { (0, (-dr2) as usize) }
                    else { continue; };

                    if l2 > grid.max_reach[p_m1][d2] {
                        continue;
                    }

                    let k1 = l1 - 1;
                    let flying1 = h_before_i - k1;
                    let h_m1 = prefix_boards[i].lens[p_m1] as usize;
                    let total_h_m1 = flying1 + h_m1;
                    if total_h_m1 < l2 {
                        continue;
                    }

                    // Hop 3: (m2_r, m2_c) -> (dest_r, dest_c)
                    let dr3 = dest_r as isize - m2_r as isize;
                    let dc3 = dest_c as isize - m2_c as isize;
                    let (d3, l3) = if dr3 == 0 && dc3 > 0 { (3, dc3 as usize) }
                    else if dr3 == 0 && dc3 < 0 { (2, (-dc3) as usize) }
                    else if dc3 == 0 && dr3 > 0 { (1, dr3 as usize) }
                    else if dc3 == 0 && dr3 < 0 { (0, (-dr3) as usize) }
                    else { continue; };

                    if l3 > grid.max_reach[p_m2][d3] {
                        continue;
                    }

                    let k2 = l2 - 1;
                    let flying2 = total_h_m1 - k2;
                    let h_m2 = prefix_boards[i].lens[p_m2] as usize;
                    let total_h_m2 = flying2 + h_m2;
                    if total_h_m2 < l3 {
                        continue;
                    }
                    let k3 = l3 - 1;

                    replacement.clear();
                    replacement.push(Op { r: op_i.r, c: op_i.c, k: k1, d: d1, l: l1 });
                    replacement.push(Op { r: m1_r, c: m1_c, k: k2, d: d2, l: l2 });
                    replacement.push(Op { r: m2_r, c: m2_c, k: k3, d: d3, l: l3 });
                    match validate_patch(&ops, &trace, i, j+1, &replacement, grid, target_cleared, deadline) {
                        CheckResult::Accept => {
                            record_patch(j+1-i, replacement.len());
                            ops.splice(i..j+1, replacement.iter().copied());
                            improved = true;
                            break 'outer;
                        }
                        CheckResult::BudgetExpired => break 'outer,
                        CheckResult::Reject => {}
                    }
                }
            }
        }
    }
    ops
}

// -------------------------------------------------------------
// 閉路・往復無駄手のブロック消去（Cycle & U-turn Pruning）
// -------------------------------------------------------------
pub fn prune_ops(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,target: usize,start: Instant,ms: u128) -> Vec<Op> {
    let deadline=deadline_at(start,ms);
    let Some(mut trace)=TraceCache::build(initial,&ops,grid,deadline) else {return ops;};
    for _ in 0..2 {
        let mut improved=false;
        for i in (0..ops.len()).rev() {
            if Instant::now()>=deadline {return ops;}
            match validate_patch(&ops,&trace,i,i+1,&[],grid,target,deadline) {
                CheckResult::Accept => {
                    ops.remove(i); record_patch(1,0); improved=true;
                    if !trace.rebuild_from(i,&ops,grid,deadline) {return ops;}
                }
                CheckResult::BudgetExpired => return ops,
                CheckResult::Reject => {}
            }
        }
        if !improved {break;}
    }
    ops
}
pub fn prune_cycle_and_uturn_ops(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,target: usize,start: Instant,ms: u128) -> Vec<Op> {
    let deadline=deadline_at(start,ms);
    let Some(mut trace)=TraceCache::build(initial,&ops,grid,deadline) else {return ops;};
    for _ in 0..2 {
        let mut accepted=false;
        'scan: for i in 0..ops.len() {
            if Instant::now()>=deadline {return ops;}
            let (p,q)=trace.endpoints[i];
            for j in i+1..(i+12).min(ops.len()) {
                let (jp,jq)=trace.endpoints[j];
                if p==jq {
                    match validate_patch(&ops,&trace,i,j+1,&[],grid,target,deadline) {
                        CheckResult::Accept => {
                            record_patch(j+1-i,0); ops.drain(i..j+1); accepted=true;
                            if !trace.rebuild_from(i,&ops,grid,deadline) {return ops;}
                            break 'scan;
                        }
                        CheckResult::BudgetExpired => return ops,
                        CheckResult::Reject => {}
                    }
                }
                if jp==q && jq==p {
                    match validate_patch(&ops,&trace,i,j+1,&ops[i+1..j],grid,target,deadline) {
                        CheckResult::Accept => {
                            record_patch(2,0); ops.remove(j); ops.remove(i); accepted=true;
                            if !trace.rebuild_from(i,&ops,grid,deadline) {return ops;}
                            break 'scan;
                        }
                        CheckResult::BudgetExpired => return ops,
                        CheckResult::Reject => {}
                    }
                }
            }
        }
        if !accepted {break;}
    }
    ops
}

// -------------------------------------------------------------
// 過去改変 第3弾: 局所再探索（LNS: Large Neighborhood Search）
// -------------------------------------------------------------
fn reopt_window(
    start_board: &BitBoard,
    target_board: &BitBoard,
    window_len: usize,
    active_cells: &[usize],
    grid: &Grid,
    max_nodes: usize,
) -> Option<Vec<Op>> {
    let dummy_params = Params::default();
    let (_, start_hash) = compute_full_eval(start_board, grid, &dummy_params);
    let (_, target_hash) = compute_full_eval(target_board, grid, &dummy_params);

    struct QNode {
        board: BitBoard,
        hash: u64,
        parent: Option<usize>,
        op: Op,
        depth: usize,
    }

    let mut arena: Vec<QNode> = Vec::with_capacity(max_nodes);
    let mut visited: FastHashMap<u64, usize> = FastHashMap::default();
    visited.insert(start_hash, 0);

    let mut queue_indices: Vec<usize> = Vec::with_capacity(max_nodes);

    arena.push(QNode {
        board: *start_board,
        hash: start_hash,
        parent: None,
        op: Op { r: 0, c: 0, k: 0, d: 0, l: 0 },
        depth: 0,
    });
    queue_indices.push(0);

    let mut head = 0;
    let mut moves_buf: Vec<Op> = Vec::with_capacity(64);

    while head < queue_indices.len() {
        let curr_id = queue_indices[head];
        head += 1;

        let curr_depth = arena[curr_id].depth;
        if curr_depth >= window_len - 1 {
            continue;
        }

        let curr_board = arena[curr_id].board;
        let curr_hash = arena[curr_id].hash;

        moves_buf.clear();
        for &p in active_cells {
            let h = curr_board.lens[p] as usize;
            if h == 0 {
                continue;
            }
            let r = p / grid.n;
            let c = p % grid.n;

            for k in 0..h {
                let jumping = h - k;
                let max_l_jump = k + 1;

                for d in 0..4 {
                    let max_reach = grid.max_reach[p][d];
                    let limit_l = max_l_jump.min(max_reach);

                    for l in 1..=limit_l {
                        let (dr, dc) = DIR[d];
                        let nr = (r as isize + dr * l as isize) as usize;
                        let nc = (c as isize + dc * l as isize) as usize;
                        let next_p = nr * grid.n + nc;

                        let g = curr_board.lens[next_p] as usize;
                        if g + jumping <= 8 {
                            moves_buf.push(Op { r, c, k, d, l });
                        }
                    }
                }
            }
        }

        for &op in &moves_buf {
            let p = op.r * grid.n + op.c;
            let (dr, dc) = DIR[op.d];
            let next_r = (op.r as isize + dr * op.l as isize) as usize;
            let next_c = (op.c as isize + dc * op.l as isize) as usize;
            let next_p = next_r * grid.n + next_c;

            let prev_p_hash = hash_cell(curr_board.colors[p], curr_board.lens[p] as usize, p, grid);
            let prev_np_hash = hash_cell(curr_board.colors[next_p], curr_board.lens[next_p] as usize, next_p, grid);

            let mut next_board = curr_board;
            next_board.transfer_and_reverse(p, next_p, op.k);

            if let Some(col) = grid.nest_color_at[p] {
                next_board.clean_nest(p, col);
            }
            if let Some(col) = grid.nest_color_at[next_p] {
                next_board.clean_nest(next_p, col);
            }

            if next_board.lens[p] == 0 {
                next_board.clear_occupied(p);
            }
            if next_board.lens[next_p] > 0 {
                next_board.set_occupied(next_p);
            } else {
                next_board.clear_occupied(next_p);
            }

            let next_p_hash = hash_cell(next_board.colors[p], next_board.lens[p] as usize, p, grid);
            let next_np_hash = hash_cell(next_board.colors[next_p], next_board.lens[next_p] as usize, next_p, grid);

            let next_hash = curr_hash ^ prev_p_hash ^ prev_np_hash ^ next_p_hash ^ next_np_hash;

            if next_hash == target_hash {
                if next_board.occupied_mask == target_board.occupied_mask
                    && next_board.lens == target_board.lens
                    && next_board.colors == target_board.colors
                {
                    let mut path = Vec::new();
                    path.push(op);
                    let mut c_node = curr_id;
                    while let Some(parent_id) = arena[c_node].parent {
                        path.push(arena[c_node].op);
                        c_node = parent_id;
                    }
                    path.reverse();
                    return Some(path);
                }
            }

            if let Some(&prev_d) = visited.get(&next_hash) {
                if prev_d <= curr_depth + 1 {
                    continue;
                }
            }

            if arena.len() >= max_nodes {
                return None;
            }

            visited.insert(next_hash, curr_depth + 1);
            let new_node_id = arena.len();
            arena.push(QNode {
                board: next_board,
                hash: next_hash,
                parent: Some(curr_id),
                op,
                depth: curr_depth + 1,
            });
            queue_indices.push(new_node_id);
        }
    }

    None
}

// -------------------------------------------------------------
// 踏み台の過去挿入: 余剰スライムの踏み台化（Drop-off 最適化）
// -------------------------------------------------------------
fn optimize_trampoline_dropoff(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
) -> Vec<Op> {
    let mut i = 0;
    while i < ops.len() {
        let op = ops[i];
        if op.k == 0 && op.l <= 2 {
            let mut test_ops = ops.clone();
            test_ops[i].k = 1;
            if let Some(cleared) = simulate_all(*initial_board, &test_ops, grid) {
                if cleared >= target_cleared {
                    ops = test_ops;
                }
            }
        }
        i += 1;
    }
    ops
}

pub fn lns_optimize(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    start_time: Instant,
    deadline_ms: u128,
) -> Vec<Op> {
    let window_sizes = [3, 4, 5];

    let mut boards = Vec::with_capacity(ops.len() + 1);
    boards.push(*initial_board);
    let mut cur = *initial_board;
    for &op in &ops {
        if apply_op(&mut cur, op, grid).is_some() {
            boards.push(cur);
        } else {
            return ops;
        }
    }

    for &w in &window_sizes {
        if start_time.elapsed().as_millis() >= deadline_ms {
            break;
        }

        let mut start = 0;
        while start + w <= ops.len() {
            if start_time.elapsed().as_millis() >= deadline_ms {
                break;
            }

            let start_board = boards[start];
            let target_board = boards[start + w];

            let mut active_cells = Vec::with_capacity(w * 2 + 8);
            for op in &ops[start..(start + w)] {
                let p = op.r * grid.n + op.c;
                let (dr, dc) = DIR[op.d];
                let nr = (op.r as isize + dr * op.l as isize) as usize;
                let nc = (op.c as isize + dc * op.l as isize) as usize;
                let np = nr * grid.n + nc;
                if !active_cells.contains(&p) {
                    active_cells.push(p);
                }
                if !active_cells.contains(&np) {
                    active_cells.push(np);
                }
            }

            // 踏み台候補（Trampoline Candidates）の追加:
            // 直線射程内にあるスライムのいる床マスを踏み台として巻き込む
            let orig_len = active_cells.len();
            for idx in 0..orig_len {
                let p = active_cells[idx];
                let r = p / grid.n;
                let c = p % grid.n;
                for d in 0..4 {
                    let reach = grid.max_reach[p][d];
                    let (dr, dc) = DIR[d];
                    for l in 1..=reach.min(3) {
                        let nr = (r as isize + dr * l as isize) as usize;
                        let nc = (c as isize + dc * l as isize) as usize;
                        let np = nr * grid.n + nc;
                        if start_board.lens[np] > 0 && !active_cells.contains(&np) {
                            active_cells.push(np);
                            break;
                        }
                    }
                }
                if active_cells.len() >= 8 {
                    break;
                }
            }

            if let Some(new_ops) = reopt_window(&start_board, &target_board, w, &active_cells, grid, 3000) {
                if new_ops.len() < w {
                    ops.splice(start..(start + w), new_ops);

                    boards.truncate(start + 1);
                    let mut b = boards[start];
                    for &op in &ops[start..] {
                        if apply_op(&mut b, op, grid).is_some() {
                            boards.push(b);
                        }
                    }
                    continue;
                }
            }

            start += 1;
        }
    }

    ops
}

// -------------------------------------------------------------
// 3手先・合流連鎖ジャンプの高速先読み評価（Quiescence Lookahead）
// -------------------------------------------------------------
#[inline(always)]
fn evaluate_lookahead(
    board: &BitBoard,
    pos: usize,
    grid: &Grid,
    depth_left: usize,
) -> i64 {
    if depth_left == 0 || board.lens[pos] == 0 {
        return 0;
    }

    let h = board.lens[pos] as usize;
    let r = pos / grid.n;
    let c = pos % grid.n;

    let mut best_gain = 0i64;

    for k in 0..h {
        let jumping = h - k;
        let max_l_jump = k + 1;

        for d in 0..4 {
            let max_reach = grid.max_reach[pos][d];
            let limit_l = max_l_jump.min(max_reach);

            for l in 1..=limit_l {
                let (dr, dc) = DIR[d];
                let nr = (r as isize + dr * l as isize) as usize;
                let nc = (c as isize + dc * l as isize) as usize;
                let next_p = nr * grid.n + nc;

                let g = board.lens[next_p] as usize;
                if g + jumping > 8 {
                    continue;
                }

                let mut b2 = *board;
                b2.transfer_and_reverse(pos, next_p, k);
                let mut cleared = 0;
                if let Some(col) = grid.nest_color_at[pos] {
                    cleared += b2.clean_nest(pos, col);
                }
                if let Some(col) = grid.nest_color_at[next_p] {
                    cleared += b2.clean_nest(next_p, col);
                }

                let mut gain = 0i64;
                if cleared > 0 {
                    gain += (cleared as i64) * 8_000 + (cleared as i64) * (cleared as i64) * 4_000;
                } else if g > 0 {
                    gain += 1_500;
                    let dest_top_col = ((board.colors[next_p] >> ((g - 1) * 4)) & 0xF) as usize;
                    let jump_top_col = ((board.colors[pos] >> ((h - 1) * 4)) & 0xF) as usize;
                    if dest_top_col == jump_top_col {
                        gain += 2_000;
                    }
                    if depth_left > 1 {
                        gain += evaluate_lookahead(&b2, next_p, grid, depth_left - 1);
                    }
                }

                if gain > best_gain {
                    best_gain = gain;
                }
            }
        }
    }

    best_gain
}

// -------------------------------------------------------------
// メイン探索関数: 極限高速化 Chokudai Search
// -------------------------------------------------------------
const SEARCH_TIME_LIMIT_MS: u128 = 1350;
const TOTAL_TIME_LIMIT_MS: u128 = 1750;

fn chokudai_search(initial_board: BitBoard, grid: &Grid, orig_params: &Params) -> Vec<Op> {
    let start_time = Instant::now();

    // スライム数に応じた適応的パラメータ調整
    let mut local_params = *orig_params;
    if grid.initial_slimes <= 75 {
        local_params.w_nest = 1800;
        local_params.w_color_run_quad = 200;
        local_params.w_pure_tower = 0;
        local_params.w_line_of_sight = local_params.w_line_of_sight.min(1000);
        local_params.w_top_match = local_params.w_top_match.min(1200);
        local_params.w_nest_block = local_params.w_nest_block.min(1200);
    }
    let params = &local_params;

    let is_large_dense = grid.initial_slimes > 75;
    let (merge_base, connect_bonus, common_bonus, retreat_factor, batch_bonus_factor) = if is_large_dense {
        (0i64, 0i64, 0i64, 0i64, 0i64)
    } else {
        (1100i64, 1600i64, 800i64, 700i64, 2_000_000i64)
    };

    let max_depth = params.max_depth;
    let beam_width = params.beam_width;
    let max_candidates = params.max_cand;

    let mut arena: Vec<Node> = Vec::with_capacity(400000);
    let mut beam: Vec<Vec<usize>> = vec![Vec::new(); max_depth + 1];
    let mut beam_idx: Vec<usize> = vec![0; max_depth + 1];
    let mut visited: FastHashMap<u64, usize> = FastHashMap::default();

    let (initial_static_score, initial_hash) = compute_full_eval(&initial_board, grid, params);
    let initial_slimes = grid.initial_slimes;

    arena.push(Node {
        board: initial_board,
        parent: None,
        op: None,
        depth: 0,
        static_score: initial_static_score,
        final_score: initial_static_score,
        slimes_left: initial_slimes,
        hash: initial_hash,
    });
    beam[0].push(0);
    visited.insert(initial_hash, 0);

    let mut best_goal_node: Option<usize> = None;
    let mut best_any_node: usize = 0;

    // 再利用バッファ（ゼロアロケーション）
    let mut moves_buf: Vec<Op> = Vec::with_capacity(128);
    let mut cand_list: Vec<(i64, i64, Op, BitBoard, u64, usize)> = Vec::with_capacity(128);

    while start_time.elapsed().as_millis()
        < if best_goal_node.is_some() {
            SEARCH_TIME_LIMIT_MS
        } else {
            TOTAL_TIME_LIMIT_MS
        }
        && arena.len() < if best_goal_node.is_some() { 380_000 } else { 450_000 }
    {
        let mut expanded_any = false;

        for d in 0..max_depth {
            if start_time.elapsed().as_millis()
                >= if best_goal_node.is_some() {
                    SEARCH_TIME_LIMIT_MS
                } else {
                    TOTAL_TIME_LIMIT_MS
                }
                || arena.len() >= if best_goal_node.is_some() { 380_000 } else { 450_000 }
            {
                break;
            }

            if beam_idx[d] < beam[d].len() {
                let curr_id = beam[d][beam_idx[d]];
                beam_idx[d] += 1;
                expanded_any = true;

                if arena[curr_id].slimes_left == 0 {
                    continue;
                }

                let curr_board = arena[curr_id].board;
                let last_op = arena[curr_id].op;
                let curr_hash = arena[curr_id].hash;
                let curr_static_score = arena[curr_id].static_score;
                let curr_slimes = arena[curr_id].slimes_left;

                // ゼロアロケーションで合法手を生成
                generate_valid_moves_into(&curr_board, last_op, grid, &mut moves_buf);

                cand_list.clear();

                for &op in &moves_buf {
                    let p = op.r * grid.n + op.c;
                    let (dr, dc) = DIR[op.d];
                    let next_r = (op.r as isize + dr * op.l as isize) as usize;
                    let next_c = (op.c as isize + dc * op.l as isize) as usize;
                    let next_p = next_r * grid.n + next_c;

                    let is_merge = curr_board.lens[next_p] > 0;

                    let prev_p_score =
                        eval_cell(curr_board.colors[p], curr_board.lens[p] as usize, p, grid, params);
                    let prev_np_score = eval_cell(
                        curr_board.colors[next_p],
                        curr_board.lens[next_p] as usize,
                        next_p,
                        grid,
                        params,
                    );
                    let prev_p_hash =
                        hash_cell(curr_board.colors[p], curr_board.lens[p] as usize, p, grid);
                    let prev_np_hash = hash_cell(
                        curr_board.colors[next_p],
                        curr_board.lens[next_p] as usize,
                        next_p,
                        grid,
                    );

                    let mut next_board = curr_board;
                    next_board.transfer_and_reverse(p, next_p, op.k);

                    let mut cleared = 0;
                    if let Some(col) = grid.nest_color_at[p] {
                        cleared += next_board.clean_nest(p, col);
                    }
                    if let Some(col) = grid.nest_color_at[next_p] {
                        cleared += next_board.clean_nest(next_p, col);
                    }

                    if next_board.lens[p] == 0 {
                        next_board.clear_occupied(p);
                    }
                    if next_board.lens[next_p] > 0 {
                        next_board.set_occupied(next_p);
                    } else {
                        next_board.clear_occupied(next_p);
                    }

                    let next_p_score =
                        eval_cell(next_board.colors[p], next_board.lens[p] as usize, p, grid, params);
                    let next_np_score = eval_cell(
                        next_board.colors[next_p],
                        next_board.lens[next_p] as usize,
                        next_p,
                        grid,
                        params,
                    );
                    let next_p_hash =
                        hash_cell(next_board.colors[p], next_board.lens[p] as usize, p, grid);
                    let next_np_hash = hash_cell(
                        next_board.colors[next_p],
                        next_board.lens[next_p] as usize,
                        next_p,
                        grid,
                    );

                    let next_hash = curr_hash ^ prev_p_hash ^ prev_np_hash ^ next_p_hash ^ next_np_hash;

                    if let Some(&prev_d) = visited.get(&next_hash) {
                        if prev_d <= d + 1 {
                            continue;
                        }
                    }

                    let mut bonus = 0i64;
                    if is_merge {
                        // 【A: 合流・拾い上げボーナス】荷物（occupied cell）の集約
                        bonus += merge_base;

                        let dest_h = curr_board.lens[next_p] as usize;
                        let dest_top_col = ((curr_board.colors[next_p] >> ((dest_h - 1) * 4)) & 0xF) as usize;
                        let jump_top_col = ((curr_board.colors[p] >> ((curr_board.lens[p] as usize - 1) * 4)) & 0xF) as usize;

                        // 【C: 同色接合引力】接合面で同色が繋がる
                        if dest_top_col == jump_top_col {
                            bonus += connect_bonus;
                        }

                        // 【C: 同色引力】ジャンプ側と着地側で共有する同色の引力ボーナス
                        let mut jump_colors_mask = 0u32;
                        let mut temp_jc = curr_board.colors[p] >> (op.k * 4);
                        for _ in op.k..curr_board.lens[p] as usize {
                            jump_colors_mask |= 1 << (temp_jc & 0xF);
                            temp_jc >>= 4;
                        }
                        let mut dest_colors_mask = 0u32;
                        let mut temp_dc = curr_board.colors[next_p];
                        for _ in 0..dest_h {
                            dest_colors_mask |= 1 << (temp_dc & 0xF);
                            temp_dc >>= 4;
                        }
                        let common_colors = (jump_colors_mask & dest_colors_mask).count_ones() as i64;
                        bonus += common_colors * common_bonus;

                        // 【新: 3手先・合流連鎖ジャンプの高速先読みボーナス】
                        if !is_large_dense {
                            let lookahead_gain = evaluate_lookahead(&next_board, next_p, grid, 2);
                            bonus += lookahead_gain;
                        }
                    } else if cleared == 0 {
                        // 【B: 空きマス移動時の後退ペナルティ（ほどほどに抑制）】
                        let h = curr_board.lens[p] as usize;
                        let mut sum_diff = 0i32;
                        let mut temp_c = curr_board.colors[p] >> (op.k * 4);
                        for _ in op.k..h {
                            let col = (temp_c & 0xF) as usize;
                            let d_old = grid.get_dist(p, grid.nest_pos[col]) as i32;
                            let d_new = grid.get_dist(next_p, grid.nest_pos[col]) as i32;
                            sum_diff += d_new - d_old;
                            temp_c >>= 4;
                        }
                        // 完全根絶ではなく、距離増加量に応じた適度なペナルティ
                        if sum_diff > 0 {
                            bonus -= (sum_diff as i64) * retreat_factor;
                        }
                    }

                    // 【D: 一括帰巣ボーナス（タワーにまとめて一気に巣に入れる）】
                    if cleared >= 2 && batch_bonus_factor > 0 {
                        bonus += (cleared as i64) * (cleared as i64) * batch_bonus_factor;
                    }

                    let next_static_score =
                        curr_static_score - prev_p_score - prev_np_score + next_p_score + next_np_score + bonus;
                    let next_slimes = curr_slimes - cleared;
                    let cleared_total = grid.initial_slimes - next_slimes;
                    let next_final_score = next_static_score
                        + (cleared_total as i64) * 100_000_000
                        - ((d + 1) as i64) * params.w_depth;

                    cand_list.push((
                        next_final_score,
                        next_static_score,
                        op,
                        next_board,
                        next_hash,
                        next_slimes,
                    ));
                }

                cand_list.sort_unstable_by(|a, b| b.0.cmp(&a.0));

                for &(final_score, static_score, op, cand_board, cand_hash, next_slimes) in
                    cand_list.iter().take(max_candidates)
                {
                    visited.insert(cand_hash, d + 1);

                    let new_id = arena.len();
                    arena.push(Node {
                        board: cand_board,
                        parent: Some(curr_id),
                        op: Some(op),
                        depth: d + 1,
                        static_score,
                        final_score,
                        slimes_left: next_slimes,
                        hash: cand_hash,
                    });

                    if next_slimes == 0 {
                        if best_goal_node.is_none()
                            || d + 1 < arena[best_goal_node.unwrap()].depth
                        {
                            best_goal_node = Some(new_id);
                        }
                    }

                    let cur_best = &arena[best_any_node];
                    if next_slimes < cur_best.slimes_left
                        || (next_slimes == cur_best.slimes_left && final_score > cur_best.final_score)
                    {
                        best_any_node = new_id;
                    }

                    insert_into_beam(&mut beam[d + 1], new_id, &arena, beam_width);
                }
            }
        }

        if !expanded_any {
            break;
        }
    }

    // 解の復元
    let target_node = best_goal_node.unwrap_or(best_any_node);
    let mut raw_ans = Vec::new();
    let mut curr = Some(target_node);
    while let Some(node_id) = curr {
        if let Some(op) = arena[node_id].op {
            raw_ans.push(op);
        }
        curr = arena[node_id].parent;
    }
    raw_ans.reverse();

    let final_slimes_left = arena[target_node].slimes_left;
    let target_cleared = grid.initial_slimes - final_slimes_left;
    let raw_len = raw_ans.len();

    // 過去改変 パイプライン（第1弾: 不要手削除 + 第2弾: 直線マージ + 踏み台挿入 + 第3弾: 局所再探索 LNS）
    let mut ans = prune_ops(raw_ans, &initial_board, grid, target_cleared, start_time, TOTAL_TIME_LIMIT_MS);
    for _pass in 0..3 {
        let before_len = ans.len();
        ans = merge_straight_jumps(ans, &initial_board, grid, target_cleared, start_time, TOTAL_TIME_LIMIT_MS);
        ans = prune_ops(ans, &initial_board, grid, target_cleared, start_time, TOTAL_TIME_LIMIT_MS);
        if ans.len() == before_len {
            break;
        }
    }
    // 踏み台の過去挿入: 余剰スライムの踏み台化
    ans = optimize_trampoline_dropoff(ans, &initial_board, grid, target_cleared);

    // 時間いっぱい（TOTAL_TIME_LIMIT_MS: 1750msまで）LNS と Pruning を反復最適化！
    while start_time.elapsed().as_millis() < TOTAL_TIME_LIMIT_MS {
        let before_len = ans.len();
        ans = lns_optimize(ans, &initial_board, grid, start_time, TOTAL_TIME_LIMIT_MS);
        ans = prune_ops(ans, &initial_board, grid, target_cleared, start_time, TOTAL_TIME_LIMIT_MS);
        ans = merge_straight_jumps(ans, &initial_board, grid, target_cleared, start_time, TOTAL_TIME_LIMIT_MS);
        ans = prune_ops(ans, &initial_board, grid, target_cleared, start_time, TOTAL_TIME_LIMIT_MS);
        if ans.len() == before_len {
            break;
        }
    }
    let mut pruned_len = ans.len();

    // 100%完全クリア保証フォールバック (Greedy Clean-up)
    if final_slimes_left > 0 {
        let mut sim_board = initial_board;
        for &op in &ans {
            apply_op(&mut sim_board, op, grid);
        }
        let n = grid.n;
        let mut step = 0;
        while sim_board.lens.iter().map(|&l| l as usize).sum::<usize>() > 0 && step < 1000 {
            step += 1;
            let mut moved = false;
            for p in 0..n * n {
                let h = sim_board.lens[p] as usize;
                if h == 0 { continue; }
                let color = ((sim_board.colors[p] >> ((h - 1) * 4)) & 0xF) as usize;
                let nest = grid.nest_pos[color];
                if p == nest { continue; }

                let d_cur = grid.dist[p * 400 + nest];
                for dir in 0..4 {
                    if grid.max_reach[p][dir] == 0 { continue; }
                    let (dr, dc) = DIR[dir];
                    let r = (p / n) as isize + dr;
                    let c = (p % n) as isize + dc;
                    let next_p = r as usize * n + c as usize;
                    if sim_board.lens[next_p] + 1 <= 8 {
                        let d_next = grid.dist[next_p * 400 + nest];
                        if d_next < d_cur {
                            let op = Op {
                                r: p / n,
                                c: p % n,
                                k: h - 1,
                                d: dir,
                                l: 1,
                            };
                            apply_op(&mut sim_board, op, grid);
                            ans.push(op);
                            moved = true;
                            break;
                        }
                    }
                }
                if moved { break; }
            }
            if !moved {
                for p in 0..n * n {
                    let h = sim_board.lens[p] as usize;
                    if h == 0 { continue; }
                    for dir in 0..4 {
                        if grid.max_reach[p][dir] >= 1 {
                            let (dr, dc) = DIR[dir];
                            let r = (p / n) as isize + dr;
                            let c = (p % n) as isize + dc;
                            let next_p = r as usize * n + c as usize;
                            if sim_board.lens[next_p] + 1 <= 8 {
                                let op = Op {
                                    r: p / n,
                                    c: p % n,
                                    k: h - 1,
                                    d: dir,
                                    l: 1,
                                };
                                apply_op(&mut sim_board, op, grid);
                                ans.push(op);
                                moved = true;
                                break;
                            }
                        }
                    }
                    if moved { break; }
                }
            }
            if !moved { break; }
        }
        ans = prune_ops(ans, &initial_board, grid, grid.initial_slimes, start_time, TOTAL_TIME_LIMIT_MS);
        pruned_len = ans.len();
    }

    let slimes_remaining = {
        let mut check_board = initial_board;
        for &op in &ans {
            apply_op(&mut check_board, op, grid);
        }
        check_board.lens.iter().map(|&l| l as usize).sum::<usize>()
    };

    let final_abs_score = ans.len() + 100_000 * slimes_remaining;

    eprintln!(
        "STAT: nodes={} best_score={} slimes_left={} moves={} raw_moves={} pruned={} elapsed_ms={}",
        arena.len(),
        final_abs_score,
        slimes_remaining,
        ans.len(),
        raw_len,
        raw_len - pruned_len,
        start_time.elapsed().as_millis()
    );

    ans
}

fn parse_args() -> Params {
    let mut params = Params::default();
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--w_tower_quad" if i + 1 < args.len() => {
                params.w_tower_quad = args[i + 1].parse().unwrap_or(params.w_tower_quad);
                i += 2;
            }
            "--w_tower_linear" if i + 1 < args.len() => {
                params.w_tower_linear = args[i + 1].parse().unwrap_or(params.w_tower_linear);
                i += 2;
            }
            "--w_pair" if i + 1 < args.len() => {
                params.w_pair = args[i + 1].parse().unwrap_or(params.w_pair);
                i += 2;
            }
            "--w_nest" if i + 1 < args.len() => {
                params.w_nest = args[i + 1].parse().unwrap_or(params.w_nest);
                i += 2;
            }
            "--w_purity" if i + 1 < args.len() => {
                params.w_purity = args[i + 1].parse().unwrap_or(params.w_purity);
                i += 2;
            }
            "--w_mix" if i + 1 < args.len() => {
                params.w_mix = args[i + 1].parse().unwrap_or(params.w_mix);
                i += 2;
            }
            "--w_depth" if i + 1 < args.len() => {
                params.w_depth = args[i + 1].parse().unwrap_or(params.w_depth);
                i += 2;
            }
            "--w_line_of_sight" if i + 1 < args.len() => {
                params.w_line_of_sight = args[i + 1].parse().unwrap_or(params.w_line_of_sight);
                i += 2;
            }
            "--w_top_match" if i + 1 < args.len() => {
                params.w_top_match = args[i + 1].parse().unwrap_or(params.w_top_match);
                i += 2;
            }
            "--w_nest_block" if i + 1 < args.len() => {
                params.w_nest_block = args[i + 1].parse().unwrap_or(params.w_nest_block);
                i += 2;
            }
            "--w_color_run_quad" if i + 1 < args.len() => {
                params.w_color_run_quad = args[i + 1].parse().unwrap_or(params.w_color_run_quad);
                i += 2;
            }
            "--w_color_transition" if i + 1 < args.len() => {
                params.w_color_transition = args[i + 1].parse().unwrap_or(params.w_color_transition);
                i += 2;
            }
            "--w_pure_tower" if i + 1 < args.len() => {
                params.w_pure_tower = args[i + 1].parse().unwrap_or(params.w_pure_tower);
                i += 2;
            }
            "--beam_width" if i + 1 < args.len() => {
                params.beam_width = args[i + 1].parse().unwrap_or(params.beam_width);
                i += 2;
            }
            "--max_cand" if i + 1 < args.len() => {
                params.max_cand = args[i + 1].parse().unwrap_or(params.max_cand);
                i += 2;
            }
            "--max_depth" if i + 1 < args.len() => {
                params.max_depth = args[i + 1].parse().unwrap_or(params.max_depth);
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    params
}

// -------------------------------------------------------------
// メイン関数
// -------------------------------------------------------------
pub fn solve(input_str: &str) {
    let start_time = Instant::now();
    const B01_DEADLINE_MS: u128 = 1750;
    let params = parse_args();

    let mut lines = input_str.lines().filter(|l| !l.trim().is_empty());

    let first_line = lines.next().unwrap();
    let mut tokens = first_line.split_whitespace();
    let n: usize = tokens.next().unwrap().parse().unwrap();
    let k: usize = tokens.next().unwrap().parse().unwrap();

    let mut raw_grid = Vec::with_capacity(n);
    for _ in 0..n {
        let line = lines.next().unwrap().trim();
        raw_grid.push(line.chars().collect::<Vec<char>>());
    }

    let (grid, initial_board) = Grid::new(n, k, &raw_grid);

    // =========================================================
    // 超密集盤面へのマクロ樹状合流の注入
    // =========================================================
    let dir_step: [isize; 4] = [-(n as isize), n as isize, -1, 1];
    let mut floor = vec![true; n * n];
    let mut init_stacks: Vec<Vec<usize>> = vec![Vec::new(); n * n];
    let mut plates = vec![0; k];

    for i in 0..n {
        for (j, &ch) in raw_grid[i].iter().enumerate() {
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

    let max_ray = crate::a44::compute_max_ray(n, &floor);
    let dist_table = crate::a44::compute_all_pairs_dist(n, &floor);
    let mut a44_board = crate::a44::BitBoard::new(n, &init_stacks);
    let mut ctx = Box::new(crate::a44::BfsContext::new());
    let mut prefix_ops_a44 = Vec::new();
    let floor_cells = floor.iter().filter(|&&f| f).count();
    let fill_ratio = grid.initial_slimes as f64 / floor_cells as f64;
    let allow_pre_cluster = fill_ratio < 0.60;

    if allow_pre_cluster {
        for color in 0..k {
            crate::a44::pre_cluster_dense(
                n,
                &dir_step,
                &max_ray,
                &mut a44_board,
                &plates,
                &mut prefix_ops_a44,
                color,
                &dist_table,
                &mut ctx,
            );
        }
    }

    let prefix_ops: Vec<Op> = prefix_ops_a44
        .into_iter()
        .map(|op| {
            let d = match op.d {
                'U' => 0,
                'D' => 1,
                'L' => 2,
                'R' => 3,
                _ => 0,
            };
            Op {
                r: op.r,
                c: op.c,
                k: op.k,
                d,
                l: op.l,
            }
        })
        .collect();

    let mut clustered_board = initial_board;
    let mut valid_prefix = true;
    for &op in &prefix_ops {
        if apply_op(&mut clustered_board, op, &grid).is_none() {
            valid_prefix = false;
            break;
        }
    }

    let (start_board, applied_prefix_ops) = if valid_prefix {
        (clustered_board, prefix_ops)
    } else {
        (initial_board, Vec::new())
    };

    let search_ops = chokudai_search(start_board, &grid, &params);
    let mut all_ops = applied_prefix_ops;
    all_ops.extend(search_ops);
    let mut cur_ops = prune_ops(all_ops, &initial_board, &grid, grid.initial_slimes, start_time, B01_DEADLINE_MS);
    for _ in 0..3 {
        let before_len = cur_ops.len();
        cur_ops = merge_straight_jumps(cur_ops, &initial_board, &grid, grid.initial_slimes, start_time, B01_DEADLINE_MS);
        cur_ops = shortcut_ray_jumps(cur_ops, &initial_board, &grid, grid.initial_slimes, start_time, B01_DEADLINE_MS);
        cur_ops = prune_ops(cur_ops, &initial_board, &grid, grid.initial_slimes, start_time, B01_DEADLINE_MS);
        if cur_ops.len() == before_len {
            break;
        }
    }
    let final_ops = cur_ops;

    for op in final_ops {
        println!("{} {} {} {} {}", op.r, op.c, op.k, DIR_CHAR[op.d], op.l);
    }
}

}


fn main() {
    let execution_start = std::time::Instant::now();
    let global_params = parse_global_args();
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut slimes = 0;
    let mut walls = 0;
    for line in input.lines().filter(|l| !l.trim().is_empty()).skip(1) {
        for ch in line.chars() {
            if ch.is_ascii_lowercase() {
                slimes += 1;
            } else if ch == '#' {
                walls += 1;
            }
        }
    }

    let use_a44 = (slimes <= 120) || (slimes <= 145 && walls < 100);

    eprintln!("BRANCH: {}", if use_a44 {"A44"} else {"B01"});
    if use_a44 {
        a44::solve(&input, &global_params);
    } else {
        b01::solve(&input);
    }
    b01::print_patch_stats();
    eprintln!("TIME: elapsed_ms={:.3}",execution_start.elapsed().as_secs_f64()*1000.0);
}
