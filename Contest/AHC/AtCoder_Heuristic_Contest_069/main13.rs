use std::collections::{BinaryHeap, VecDeque};
use std::io::{self, BufWriter, Write};
use std::time::Instant;

const MAXN: usize = 50;
const DR: [i32; 4] = [-1, 1, 0, 0];
const DC: [i32; 4] = [0, 0, -1, 1];

// パラメータをまとめた構造体
#[derive(Clone)]
struct Params {
    w_adj: f64,
    w_time: f64,
    w_expose: f64,
    w_zone: f64,
    w_capacity: f64,
    w_pipe: f64,
    w_suffocate: f64,
    w_pocket: f64,
    base_cost_weight: f64,
    t_mm: f64,
    t_deep: f64,
    max_t_base: f64,
    
    // --- 新規追加：MultiMoveパラメータ ---
    mm_compact_th1: f64,
    mm_v_th: f64,
    mm_compact_th2: f64,
    mm_density_th: f64,
    mm_density_bonus_th: f64,
    mm_v_bonus_th: f64,
    mm_inter_bonus1: usize,
    mm_inter_bonus2: usize,
    density_reject_mult: f64,
    p3_delta_weight: f64,
    p3_bbox_area_mult: f64,
    p3_bbox_aspect_mult: f64,
    p3_lookahead_steps: usize,
    w_adj_pond: f64,
    w_adj_border: f64,
}

#[derive(Clone, Default)]
struct Group {
    p: usize,
    s: i64,
    t: i64,
    v: i64,
    c_min: f64,
    cells: Vec<(usize, usize)>,
    active: bool,
    phase_id: usize,
}

#[derive(Clone)]
struct PlaceResult {
    cells: Vec<(usize, usize)>,
    compact: f64,
    adj: i32,
    adj_pond: i32,
    adj_border: i32,
    tie_breaker: f64,
    time_bonus: f64,
    exposed_free: i32,
    max_free_rect: i32,
    pipe_bonus: f64,
    suffocate_penalty: f64,
    pocket_fit: f64,
}

#[derive(Default)]
struct Diag {
    accepted: i32,
    rejected: i32,
    total_compact: f64,
    phase1_count: i32,
    phase2_count: i32,
    phase25_count: i32,
    phase3_count: i32,
    phase1_compact: f64,
    phase2_compact: f64,
    phase25_compact: f64,
    phase3_compact: f64,

    phase1_fee: i64,
    phase2_fee: i64,
    phase25_fee: i64,
    phase3_fee: i64,

    total_fee: i64,
    total_move_cost: i64,
    move_count: i32,
    bucket_n: [i32; 4],
    bucket_compact_sum: [f64; 4],
    bucket_fee: [i64; 4],
    bucket_reject: [i32; 4],

    time_phase1_us: u128,
    time_phase2_us: u128,
    time_phase25_us: u128,
    time_phase3_us: u128,
    time_eval_cap_us: u128,
    time_multimove_us: u128,
    occupancy_history: [f64; 10],
}

fn size_bucket(p: usize) -> usize {
    if p <= 30 {
        0
    } else if p <= 70 {
        1
    } else if p <= 110 {
        2
    } else {
        3
    }
}

#[derive(Clone)]
struct MultiMovePlan {
    icells: Vec<(usize, usize)>,
    net_gain: i64,
    intersecting: Vec<usize>,
    move_plans: Vec<(usize, Vec<(usize, usize)>)>,
}

#[derive(Copy, Clone, PartialEq)]
struct MinDistState {
    dist: f64,
    x: usize,
    y: usize,
}
impl Eq for MinDistState {}
impl Ord for MinDistState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .dist
            .total_cmp(&self.dist)
            .then_with(|| other.x.cmp(&self.x))
            .then_with(|| other.y.cmp(&self.y))
    }
}
impl PartialOrd for MinDistState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone, PartialEq)]
struct MinCostState {
    cost: f64,
    x: usize,
    y: usize,
}
impl Eq for MinCostState {}
impl Ord for MinCostState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.x.cmp(&self.x))
            .then_with(|| other.y.cmp(&self.y))
    }
}
impl PartialOrd for MinCostState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct Solver {
    n: usize,
    r: f64,
    grid: [[char; MAXN]; MAXN],
    own: [[i32; MAXN]; MAXN],
    grp: Vec<Group>,
    start_time: Instant,
    bps: [[i32; MAXN + 1]; MAXN + 1],
    ponds_bps: [[i32; MAXN + 1]; MAXN + 1],
    global_avg_rem_time: f64,
    global_avg_density: f64,
    global_total_density: f64,
    global_density_count: i32,
    global_total_rem_time: i64,
    global_groups_seen: i32,
    total_grass: i32,
    diag: Diag,
    params: Params,

    blob_stamp: u32,
    best_cost_stamp: [[u32; MAXN]; MAXN],
    best_cost_val: [[f64; MAXN]; MAXN],

    bfs_stamp: u32,
    vis_bfs_stamp: [[u32; MAXN]; MAXN],
    
    mark_grid: [[u32; MAXN]; MAXN],
    mark_stamp: u32,
}

impl Solver {
    #[inline(always)]
    fn calc_heuristics_score(
        &self,
        v: i64,
        base_cost: f64,
        comp: f64,
        adj: i32,
    adj_pond: i32,
    adj_border: i32,
        tbonus: f64,
        ef: i32,
        tb: f64,
        quick_cap: i32,
        pipe_bonus: f64,
        suff: f64,
        pocket_fit: f64,
    ) -> f64 {
        let heuristics = self.params.w_adj * adj as f64
            + self.params.w_time * tbonus
            - self.params.w_expose * ef as f64
            + self.params.w_zone * tb
            + self.params.w_capacity * (quick_cap.max(0) as f64)
            + self.params.w_pipe * pipe_bonus
            - self.params.w_suffocate * suff
            + self.params.w_pocket * pocket_fit;
        (v as f64 * comp) + (base_cost * heuristics)
    }

    fn new(n: usize, r: f64, grid: [[char; MAXN]; MAXN], total_grass: i32, start_time: Instant, params: Params) -> Self {
        Self {
            n,
            r,
            grid,
            own: [[-1; MAXN]; MAXN],
            grp: vec![Group::default(); 1000],
            start_time,
            bps: [[0; MAXN + 1]; MAXN + 1],
            ponds_bps: [[0; MAXN + 1]; MAXN + 1],
            global_avg_rem_time: 1000.0,
            global_avg_density: 0.0,
            global_total_density: 0.0,
            global_density_count: 0,
            global_total_rem_time: 0,
            global_groups_seen: 0,
            total_grass,
            diag: Diag::default(),
            params,
            blob_stamp: 0,
            best_cost_stamp: [[0; MAXN]; MAXN],
            best_cost_val: [[0.0; MAXN]; MAXN],
            bfs_stamp: 0,
            vis_bfs_stamp: [[0; MAXN]; MAXN],
            mark_grid: [[0; MAXN]; MAXN],
            mark_stamp: 0,
        }
    }
    
    fn stamp_cells(&mut self, cells: &[(usize, usize)]) -> u32 {
        self.mark_stamp = self.mark_stamp.wrapping_add(1);
        if self.mark_stamp == 0 {
            self.mark_grid = [[0; MAXN]; MAXN];
            self.mark_stamp = 1;
        }
        let s = self.mark_stamp;
        for &(x, y) in cells {
            self.mark_grid[x][y] = s;
        }
        s
    }

    fn compute_bps(&mut self) {
        self.bps = [[0; MAXN + 1]; MAXN + 1];
        for i in 1..=self.n {
            for j in 1..=self.n {
                let val = if self.grid[i - 1][j - 1] != '.' || self.own[i - 1][j - 1] != -1 {
                    1
                } else {
                    0
                };
                self.bps[i][j] =
                    val + self.bps[i - 1][j] + self.bps[i][j - 1] - self.bps[i - 1][j - 1];
            }
        }
    }

    fn init_ponds(&mut self) {
        self.ponds_bps = [[0; MAXN + 1]; MAXN + 1];
        for i in 1..=self.n {
            for j in 1..=self.n {
                let val = if self.grid[i - 1][j - 1] == '#' { 1 } else { 0 };
                self.ponds_bps[i][j] = val + self.ponds_bps[i - 1][j] + self.ponds_bps[i][j - 1]
                    - self.ponds_bps[i - 1][j - 1];
            }
        }
    }

    fn count_blocked(&self, r: usize, c: usize, h: usize, w: usize) -> i32 {
        self.bps[r + h][c + w] - self.bps[r][c + w] - self.bps[r + h][c] + self.bps[r][c]
    }

    fn count_free(&self, r: usize, c: usize, h: usize, w: usize) -> i32 {
        (h * w) as i32 - self.count_blocked(r, c, h, w)
    }

    fn count_ponds(&self, r: usize, c: usize, h: usize, w: usize) -> i32 {
        self.ponds_bps[r + h][c + w] - self.ponds_bps[r][c + w] - self.ponds_bps[r + h][c]
            + self.ponds_bps[r][c]
    }

    fn compactness(&mut self, cells: &[(usize, usize)]) -> f64 {
        let p = cells.len();
        if p == 0 {
            return 0.0;
        }
        let s = self.stamp_cells(cells);
        let mut l = 0;
        for &(x, y) in cells {
            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx < 0 || nx >= self.n as i32 || ny < 0 || ny >= self.n as i32 {
                    l += 1;
                } else if self.mark_grid[nx as usize][ny as usize] != s {
                    l += 1;
                }
            }
        }
        4.0 * (p as f64).sqrt() / l as f64
    }

    fn calc_pocket_fit(&self, cells: &[(usize, usize)]) -> f64 {
        if cells.is_empty() {
            return 0.0;
        }
        let mut pocket_score = 0.0;
        for &(x, y) in cells {
            let mut obstacle_count = 0;
            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx < 0 || nx >= self.n as i32 || ny < 0 || ny >= self.n as i32 {
                    obstacle_count += 1;
                } else if self.grid[nx as usize][ny as usize] == '#' {
                    obstacle_count += 1;
                }
            }
            if obstacle_count >= 3 {
                pocket_score += 2.0;
            } else if obstacle_count == 2 {
                pocket_score += 0.8;
            }
        }
        pocket_score / cells.len() as f64
    }

    fn calc_time_bonus(&self, cells: &[(usize, usize)], dep_time: i64) -> f64 {
        let mut tb = 0.0;
        for &(x, y) in cells {
            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx >= 0 && nx < self.n as i32 && ny >= 0 && ny < self.n as i32 {
                    let neighbor_id = self.own[nx as usize][ny as usize];
                    if neighbor_id != -1 {
                        let diff = (self.grp[neighbor_id as usize].t - dep_time).abs();
                        tb += f64::max(-1.0, 1.0 - diff as f64 / 750.0);
                    }
                }
            }
        }
        tb
    }

    fn calc_adjacency_split(&mut self, cells: &[(usize, usize)]) -> (i32, i32, i32) {
        let s = self.stamp_cells(cells);
        let mut adj_group = 0;
        let mut adj_pond = 0;
        let mut adj_border = 0;

        for &(x, y) in cells {
            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx < 0 || nx >= self.n as i32 || ny < 0 || ny >= self.n as i32 {
                    adj_border += 1;
                } else if self.mark_grid[nx as usize][ny as usize] != s {
                    let nxu = nx as usize;
                    let nyu = ny as usize;
                    if self.own[nxu][nyu] >= 0 {
                        adj_group += 1;
                    } else if self.grid[nxu][nyu] != '.' {
                        adj_pond += 1;
                    }
                }
            }
        }
        (adj_group, adj_pond, adj_border)
    }

    fn predict_future_suffocation(&self, cells: &[(usize, usize)], dep_time: i64) -> f64 {
        let mut suffocate = 0.0;
        for &(x, y) in cells {
            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx >= 0 && nx < self.n as i32 && ny >= 0 && ny < self.n as i32 {
                    let neighbor_id = self.own[nx as usize][ny as usize];
                    if neighbor_id >= 0 {
                        let nid = neighbor_id as usize;
                        if self.grp[nid].t < dep_time {
                            let dt = dep_time - self.grp[nid].t;
                            if dt > 1500 {
                                suffocate += f64::min(1.0, (dt - 1500) as f64 / 8000.0);
                            }
                        }
                    }
                }
            }
        }
        suffocate
    }

    fn clear_cells(&mut self, cells: &[(usize, usize)]) {
        for &(x, y) in cells {
            self.own[x][y] = -1;
        }
    }

    fn set_cells(&mut self, cells: &[(usize, usize)], id: i32) {
        for &(x, y) in cells {
            self.own[x][y] = id;
        }
    }

    fn clear_grp_cells(&mut self, id: usize) {
        for i in 0..self.grp[id].cells.len() {
            let (x, y) = self.grp[id].cells[i];
            self.own[x][y] = -1;
        }
    }

    fn set_grp_cells(&mut self, id: usize, target_own: i32) {
        for i in 0..self.grp[id].cells.len() {
            let (x, y) = self.grp[id].cells[i];
            self.own[x][y] = target_own;
        }
    }

    fn get_tie_breaker(
        &self,
        cells: &[(usize, usize)],
        rem_time: i64,
        dep_time: i64,
    ) -> f64 {
        if cells.is_empty() {
            return 0.0;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut touch_top = false;
        let mut touch_bottom = false;
        let mut touch_left = false;
        let mut touch_right = false;

        for &(x, y) in cells {
            cx += x as f64;
            cy += y as f64;
            if x == 0 {
                touch_top = true;
            }
            if x == self.n - 1 {
                touch_bottom = true;
            }
            if y == 0 {
                touch_left = true;
            }
            if y == self.n - 1 {
                touch_right = true;
            }
        }
        cx /= cells.len() as f64;
        cy /= cells.len() as f64;

        let dx = cx - (self.n as f64 - 1.0) / 2.0;
        let dy = cy - (self.n as f64 - 1.0) / 2.0;
        let dist = (dx * dx + dy * dy).sqrt();
        let max_dist = self.n as f64 / 1.414;
        let norm_dist = dist / max_dist;

        let time_factor = rem_time as f64 / f64::max(1.0, self.global_avg_rem_time) - 1.0;
        let mut score = time_factor * norm_dist;
        let corner_anchor = (touch_top && touch_left) as i32
            + (touch_top && touch_right) as i32
            + (touch_bottom && touch_left) as i32
            + (touch_bottom && touch_right) as i32;

        if corner_anchor > 0 {
            score += 0.20 * corner_anchor as f64;
        }

        if dep_time > 80000 {
            score += 0.08 * (dep_time - 80000) as f64 / 20000.0 * norm_dist;
        }
        score
    }

    fn calc_exposed_free(&mut self, cells: &[(usize, usize)]) -> i32 {
        let s = self.stamp_cells(cells);
        let mut exposed = 0;
        for &(x, y) in cells {
            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx >= 0
                    && nx < self.n as i32
                    && ny >= 0
                    && ny < self.n as i32
                    && self.mark_grid[nx as usize][ny as usize] != s
                {
                    if self.grid[nx as usize][ny as usize] == '.'
                        && self.own[nx as usize][ny as usize] == -1
                    {
                        exposed += 1;
                    }
                }
            }
        }
        exposed
    }

    fn calc_pipeline_bonus(&self, dep_time: i64) -> f64 {
        let mut count_near = 0;
        for j in 0..1000 {
            if self.grp[j].active && (self.grp[j].t - dep_time).abs() < 1000 {
                count_near += 1;
            }
        }
        f64::max(0.0, 1.0 - count_near as f64 / 4.0)
    }

    fn calc_max_free_rect(&self) -> i32 {
        let mut h = [0i32; MAXN];
        let mut max_score = 0;
        let mut st = [0usize; MAXN + 1];

        for i in 0..self.n {
            let row = &self.grid[i];
            let own_row = &self.own[i];
            for j in 0..self.n {
                if row[j] == '.' && own_row[j] == -1 {
                    h[j] += 1;
                } else {
                    h[j] = 0;
                }
            }
            let mut st_len = 0;
            for j in 0..=self.n {
                let val = if j < self.n { h[j] } else { 0 };
                while st_len > 0 && h[st[st_len - 1]] >= val {
                    let height = h[st[st_len - 1]];
                    st_len -= 1;
                    let width = if st_len == 0 {
                        j as i32
                    } else {
                        (j - st[st_len - 1] - 1) as i32
                    };
                    let mut score = height * width - 10 * (height - width).abs();
                    
                    if height.min(width) < 2 {
                        score -= 500;
                    }
                    
                    if score > 200 {
                        score = 200 + (score - 200) / 5;
                    }

                    if score > max_score {
                        max_score = score;
                    }
                }
                if j < self.n {
                    st[st_len] = j;
                    st_len += 1;
                }
            }
        }
        max_score
    }

    fn eval_cap(&mut self, cells: &[(usize, usize)]) -> i32 {
        let t0 = Instant::now();
        for &(x, y) in cells {
            self.own[x][y] = -2;
        }
        let cap = self.calc_max_free_rect();
        for &(x, y) in cells {
            self.own[x][y] = -1;
        }
        self.diag.time_eval_cap_us += t0.elapsed().as_micros();
        cap
    }

    fn build_box_bfs(
        &mut self,
        r: usize,
        c: usize,
        bh: usize,
        bw: usize,
        p: usize,
    ) -> Vec<(usize, usize)> {
        let cx = r as f64 + bh as f64 / 2.0;
        let cy = c as f64 + bw as f64 / 2.0;

        let mut sx = -1i32;
        let mut sy = -1i32;
        let mut best_d = 1e18;
        for i in r..r + bh {
            for j in c..c + bw {
                if self.grid[i][j] == '.' && self.own[i][j] == -1 {
                    let d = (i as f64 - cx) * (i as f64 - cx) + (j as f64 - cy) * (j as f64 - cy);
                    if d < best_d {
                        best_d = d;
                        sx = i as i32;
                        sy = j as i32;
                    }
                }
            }
        }

        if sx < 0 {
            return Vec::new();
        }
        let sx = sx as usize;
        let sy = sy as usize;

        self.bfs_stamp = self.bfs_stamp.wrapping_add(1);
        if self.bfs_stamp == 0 {
            self.vis_bfs_stamp = [[0; MAXN]; MAXN];
            self.bfs_stamp = 1;
        }
        let current_stamp = self.bfs_stamp;
        self.vis_bfs_stamp[sx][sy] = current_stamp;

        let mut pq = BinaryHeap::new();
        pq.push(MinDistState {
            dist: 0.0,
            x: sx,
            y: sy,
        });

        let mut reg = Vec::with_capacity(p);
        while let Some(MinDistState { dist: _, x, y }) = pq.pop() {
            if reg.len() >= p {
                break;
            }
            reg.push((x, y));

            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx >= r as i32
                    && nx < (r + bh) as i32
                    && ny >= c as i32
                    && ny < (c + bw) as i32
                {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.vis_bfs_stamp[nx][ny] != current_stamp
                        && self.grid[nx][ny] == '.'
                        && self.own[nx][ny] == -1
                    {
                        self.vis_bfs_stamp[nx][ny] = current_stamp;
                        let dd = (nx as f64 - cx) * (nx as f64 - cx)
                            + (ny as f64 - cy) * (ny as f64 - cy);
                        pq.push(MinDistState {
                            dist: dd,
                            x: nx,
                            y: ny,
                        });
                    }
                }
            }
        }

        if reg.len() == p {
            reg
        } else {
            Vec::new()
        }
    }

    fn build_dijkstra_blob_strict(&mut self, sx: usize, sy: usize, p: usize) -> Vec<(usize, usize)> {
        if self.grid[sx][sy] != '.' || self.own[sx][sy] != -1 {
            return Vec::new();
        }

        self.blob_stamp = self.blob_stamp.wrapping_add(1);
        if self.blob_stamp == 0 {
            self.best_cost_stamp = [[0; MAXN]; MAXN];
            self.blob_stamp = 1;
        }
        let current_stamp = self.blob_stamp;

        self.mark_stamp = self.mark_stamp.wrapping_add(1);
        if self.mark_stamp == 0 {
            self.mark_grid = [[0; MAXN]; MAXN];
            self.mark_stamp = 1;
        }
        let in_reg_stamp = self.mark_stamp;

        self.best_cost_stamp[sx][sy] = current_stamp;
        self.best_cost_val[sx][sy] = 0.0;

        let mut pq = BinaryHeap::new();
        pq.push(MinCostState {
            cost: 0.0,
            x: sx,
            y: sy,
        });

        let mut reg = Vec::with_capacity(p);
        let mut min_x = sx;
        let mut max_x = sx;
        let mut min_y = sy;
        let mut max_y = sy;

        let max_allowed_area = (p as f64 * self.params.p3_bbox_area_mult + 2.0) as usize;
        let max_aspect_diff = 3.max(((p as f64).sqrt() * self.params.p3_bbox_aspect_mult) as usize);
        let steps = self.params.p3_lookahead_steps;

        while let Some(MinCostState { cost: _, x, y }) = pq.pop() {
            if reg.len() >= p {
                break;
            }
            if self.mark_grid[x][y] == in_reg_stamp {
                continue;
            }

            self.mark_grid[x][y] = in_reg_stamp;
            reg.push((x, y));
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);

            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx >= 0 && nx < self.n as i32 && ny >= 0 && ny < self.n as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.mark_grid[nx][ny] != in_reg_stamp && self.grid[nx][ny] == '.' && self.own[nx][ny] == -1 {
                        let n_min_x = min_x.min(nx);
                        let n_max_x = max_x.max(nx);
                        let n_min_y = min_y.min(ny);
                        let n_max_y = max_y.max(ny);
                        let bbox_w = n_max_x - n_min_x + 1;
                        let bbox_h = n_max_y - n_min_y + 1;
                        let bbox_area = bbox_w * bbox_h;
                        let aspect_diff = if bbox_w > bbox_h { bbox_w - bbox_h } else { bbox_h - bbox_w };

                        if bbox_area > max_allowed_area || aspect_diff > max_aspect_diff {
                            continue;
                        }

                        let mut shared = 0;
                        for dd in 0..4 {
                            let nnx = nx as i32 + DR[dd];
                            let nny = ny as i32 + DC[dd];
                            if nnx >= 0 && nnx < self.n as i32 && nny >= 0 && nny < self.n as i32 {
                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp {
                                    shared += 1;
                                }
                            }
                        }

                        let mut diag_shared = 0;
                        let ddr = [-1, -1, 1, 1];
                        let ddc = [-1, 1, -1, 1];
                        for dd in 0..4 {
                            let nnx = nx as i32 + ddr[dd];
                            let nny = ny as i32 + ddc[dd];
                            if nnx >= 0 && nnx < self.n as i32 && nny >= 0 && nny < self.n as i32 {
                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp {
                                    diag_shared += 1;
                                }
                            }
                        }

                        // 可変ステップ数 (2手〜5手) Lookahead ボーナス計算
                        let mut max_lookahead_bonus = 0.0;
                        
                        if steps >= 2 {
                            for d2 in 0..4 {
                                let n2x = nx as i32 + DR[d2];
                                let n2y = ny as i32 + DC[d2];
                                if n2x >= 0 && n2x < self.n as i32 && n2y >= 0 && n2y < self.n as i32 {
                                    let n2x = n2x as usize;
                                    let n2y = n2y as usize;
                                    if self.mark_grid[n2x][n2y] != in_reg_stamp && (n2x != nx || n2y != ny) && self.grid[n2x][n2y] == '.' && self.own[n2x][n2y] == -1 {
                                        let mut s2 = 0;
                                        for dd in 0..4 {
                                            let nnx = n2x as i32 + DR[dd];
                                            let nny = n2y as i32 + DC[dd];
                                            if nnx >= 0 && nnx < self.n as i32 && nny >= 0 && nny < self.n as i32 {
                                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp || (nnx as usize == nx && nny as usize == ny) {
                                                    s2 += 1;
                                                }
                                            }
                                        }

                                        if steps == 2 {
                                            let total_shared_2step = shared + s2;
                                            if total_shared_2step >= 3 {
                                                let bonus = (total_shared_2step - 2) as f64 * 0.40;
                                                if bonus > max_lookahead_bonus { max_lookahead_bonus = bonus; }
                                            }
                                        } else {
                                            for d3 in 0..4 {
                                                let n3x = n2x as i32 + DR[d3];
                                                let n3y = n2y as i32 + DC[d3];
                                                if n3x >= 0 && n3x < self.n as i32 && n3y >= 0 && n3y < self.n as i32 {
                                                    let n3x = n3x as usize;
                                                    let n3y = n3y as usize;
                                                    if self.mark_grid[n3x][n3y] != in_reg_stamp && (n3x != nx || n3y != ny) && (n3x != n2x || n3y != n2y) && self.grid[n3x][n3y] == '.' && self.own[n3x][n3y] == -1 {
                                                        let mut s3 = 0;
                                                        for dd in 0..4 {
                                                            let nnx = n3x as i32 + DR[dd];
                                                            let nny = n3y as i32 + DC[dd];
                                                            if nnx >= 0 && nnx < self.n as i32 && nny >= 0 && nny < self.n as i32 {
                                                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp || (nnx as usize == nx && nny as usize == ny) || (nnx as usize == n2x && nny as usize == n2y) {
                                                                    s3 += 1;
                                                                }
                                                            }
                                                        }

                                                        if steps == 3 {
                                                            let total_shared_3step = shared + s2 + s3;
                                                            if total_shared_3step >= 4 {
                                                                let bonus = (total_shared_3step - 3) as f64 * 0.45;
                                                                if bonus > max_lookahead_bonus { max_lookahead_bonus = bonus; }
                                                            }
                                                        } else {
                                                            for d4 in 0..4 {
                                                                let n4x = n3x as i32 + DR[d4];
                                                                let n4y = n3y as i32 + DC[d4];
                                                                if n4x >= 0 && n4x < self.n as i32 && n4y >= 0 && n4y < self.n as i32 {
                                                                    let n4x = n4x as usize;
                                                                    let n4y = n4y as usize;
                                                                    if self.mark_grid[n4x][n4y] != in_reg_stamp && (n4x != nx || n4y != ny) && (n4x != n2x || n4y != n2y) && (n4x != n3x || n4y != n3y) && self.grid[n4x][n4y] == '.' && self.own[n4x][n4y] == -1 {
                                                                        let mut s4 = 0;
                                                                        for dd in 0..4 {
                                                                            let nnx = n4x as i32 + DR[dd];
                                                                            let nny = n4y as i32 + DC[dd];
                                                                            if nnx >= 0 && nnx < self.n as i32 && nny >= 0 && nny < self.n as i32 {
                                                                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp || (nnx as usize == nx && nny as usize == ny) || (nnx as usize == n2x && nny as usize == n2y) || (nnx as usize == n3x && nny as usize == n3y) {
                                                                                    s4 += 1;
                                                                                }
                                                                            }
                                                                        }

                                                                        let total_shared_4step = shared + s2 + s3 + s4;
                                                                        if total_shared_4step >= 5 {
                                                                            let bonus = (total_shared_4step - 4) as f64 * 0.50;
                                                                            if bonus > max_lookahead_bonus { max_lookahead_bonus = bonus; }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let dist = (nx as f64 - sx as f64) * (nx as f64 - sx as f64)
                            + (ny as f64 - sy as f64) * (ny as f64 - sy as f64);
                        
                        let delta_l = 4.0 - 2.0 * shared as f64 - 0.20 * diag_shared as f64 - max_lookahead_bonus;
                        let new_cost = delta_l * self.params.p3_delta_weight + dist + (bbox_area as f64 - p as f64) * 20.0;

                        let prev_cost = if self.best_cost_stamp[nx][ny] == current_stamp {
                            self.best_cost_val[nx][ny]
                        } else {
                            1e18
                        };

                        if new_cost < prev_cost {
                            self.best_cost_stamp[nx][ny] = current_stamp;
                            self.best_cost_val[nx][ny] = new_cost;
                            pq.push(MinCostState {
                                cost: new_cost,
                                x: nx,
                                y: ny,
                            });
                        }
                    }
                }
            }
        }
        if reg.len() == p { reg } else { Vec::new() }
    }

    fn build_dijkstra_blob(&mut self, sx: usize, sy: usize, p: usize) -> Vec<(usize, usize)> {
        if self.grid[sx][sy] != '.' || self.own[sx][sy] != -1 {
            return Vec::new();
        }

        self.blob_stamp = self.blob_stamp.wrapping_add(1);
        if self.blob_stamp == 0 {
            self.best_cost_stamp = [[0; MAXN]; MAXN];
            self.blob_stamp = 1;
        }
        let current_stamp = self.blob_stamp;

        self.mark_stamp = self.mark_stamp.wrapping_add(1);
        if self.mark_stamp == 0 {
            self.mark_grid = [[0; MAXN]; MAXN];
            self.mark_stamp = 1;
        }
        let in_reg_stamp = self.mark_stamp;

        self.best_cost_stamp[sx][sy] = current_stamp;
        self.best_cost_val[sx][sy] = 0.0;

        let mut pq = BinaryHeap::new();
        pq.push(MinCostState {
            cost: 0.0,
            x: sx,
            y: sy,
        });

        let mut reg = Vec::with_capacity(p);
        while let Some(MinCostState { cost: _, x, y }) = pq.pop() {
            if reg.len() >= p {
                break;
            }
            if self.mark_grid[x][y] == in_reg_stamp {
                continue;
            }

            self.mark_grid[x][y] = in_reg_stamp;
            reg.push((x, y));

            for d in 0..4 {
                let nx = x as i32 + DR[d];
                let ny = y as i32 + DC[d];
                if nx >= 0 && nx < self.n as i32 && ny >= 0 && ny < self.n as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.mark_grid[nx][ny] != in_reg_stamp && self.grid[nx][ny] == '.' && self.own[nx][ny] == -1 {
                        let mut shared = 0;
                        for dd in 0..4 {
                            let nnx = nx as i32 + DR[dd];
                            let nny = ny as i32 + DC[dd];
                            if nnx >= 0
                                && nnx < self.n as i32
                                && nny >= 0
                                && nny < self.n as i32
                            {
                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp {
                                    shared += 1;
                                }
                            }
                        }

                        let mut diag_shared = 0;
                        let ddr = [-1, -1, 1, 1];
                        let ddc = [-1, 1, -1, 1];
                        for dd in 0..4 {
                            let nnx = nx as i32 + ddr[dd];
                            let nny = ny as i32 + ddc[dd];
                            if nnx >= 0
                                && nnx < self.n as i32
                                && nny >= 0
                                && nny < self.n as i32
                            {
                                if self.mark_grid[nnx as usize][nny as usize] == in_reg_stamp {
                                    diag_shared += 1;
                                }
                            }
                        }

                        let dist = (nx as f64 - sx as f64) * (nx as f64 - sx as f64)
                            + (ny as f64 - sy as f64) * (ny as f64 - sy as f64);
                        let delta_l = 4.0 - 2.0 * shared as f64 - 0.20 * diag_shared as f64;
                        let new_cost = delta_l * self.params.p3_delta_weight + dist;

                        let prev_cost = if self.best_cost_stamp[nx][ny] == current_stamp {
                            self.best_cost_val[nx][ny]
                        } else {
                            1e18
                        };

                        if new_cost < prev_cost {
                            self.best_cost_stamp[nx][ny] = current_stamp;
                            self.best_cost_val[nx][ny] = new_cost;
                            pq.push(MinCostState {
                                cost: new_cost,
                                x: nx,
                                y: ny,
                            });
                        }
                    }
                }
            }
        }

        if reg.len() == p {
            reg
        } else {
            Vec::new()
        }
    }


    fn find_best_placement(
        &mut self,
        p: usize,
        rem_time: i64,
        dep_time: i64,
        v: i64, 
        quick_mode: bool,
        turn: usize,
        m: usize,
    ) -> PlaceResult {
        self.compute_bps();
        
        let avg_density = if self.global_density_count > 0 {
            self.global_avg_density
        } else {
            v as f64 / f64::max(1.0, p as f64 * rem_time as f64)
        };
        let base_cost = avg_density * p as f64 * rem_time as f64 * self.params.base_cost_weight;

        let mut best = PlaceResult {
            cells: Vec::new(),
            compact: -1.0,
            adj: -1,
            adj_pond: 0,
            adj_border: 0,
            tie_breaker: -1e9,
            time_bonus: -1e9,
            exposed_free: 9999,
            max_free_rect: 0,
            pipe_bonus: 0.0,
            suffocate_penalty: 0.0,
            pocket_fit: 0.0,
        };

        let get_rect_cells =
            |r: usize, c: usize, w: usize, h: usize, excess: usize| -> Vec<(usize, usize)> {
                let mut cells = Vec::with_capacity(w * h - excess);
                let h_limit = r + h - if excess > 0 { 1 } else { 0 };
                for i in r..h_limit {
                    for j in c..c + w {
                        cells.push((i, j));
                    }
                }
                if excess > 0 {
                    let last_w = w - excess;
                    for j in c..c + last_w {
                        cells.push((r + h - 1, j));
                    }
                }
                cells
            };

        let eval_approx_rect = |r: usize,
                                c: usize,
                                w: usize,
                                h: usize,
                                excess: usize,
                                dep_time: i64,
                                own: &[[i32; MAXN]; MAXN],
                                grid: &[[char; MAXN]; MAXN],
                                grp: &[Group],
                                n: usize|
         -> (i32, i32, i32, i32, f64, f64) {
            let in_rect = |nx: i32, ny: i32| -> bool {
                if nx < r as i32 || nx >= (r + h) as i32 || ny < c as i32 || ny >= (c + w) as i32 {
                    false
                } else if excess > 0 && nx == (r + h - 1) as i32 && ny >= (c + w - excess) as i32 {
                    false
                } else {
                    true
                }
            };

            let mut adj = 0;
            let mut ef = 0;
            let mut tbonus = 0.0;
            let mut suff = 0.0;

            let mut adj_pond = 0;
            let mut adj_border = 0;
            let mut eval_cell = |i: usize, j: usize| {
                for d in 0..4 {
                    let nx = i as i32 + DR[d];
                    let ny = j as i32 + DC[d];
                    if nx < 0 || nx >= n as i32 || ny < 0 || ny >= n as i32 {
                        adj_border += 1;
                    } else if !in_rect(nx, ny) {
                        let nxu = nx as usize;
                        let nyu = ny as usize;
                        if own[nxu][nyu] >= 0 {
                            adj += 1;
                            let neighbor_id = own[nxu][nyu] as usize;
                            let diff = (grp[neighbor_id].t - dep_time).abs();
                            tbonus += f64::max(-1.0, 1.0 - diff as f64 / 750.0);
                            if grp[neighbor_id].t < dep_time {
                                let dt = dep_time - grp[neighbor_id].t;
                                if dt > 1500 {
                                    suff += f64::min(1.0, (dt - 1500) as f64 / 8000.0);
                                }
                            }
                        } else if grid[nxu][nyu] != '.' {
                            adj_pond += 1;
                        } else if grid[nxu][nyu] == '.' && own[nxu][nyu] == -1 {
                            ef += 1;
                        }
                    }
                }
            };

            for j in c..c + w {
                eval_cell(r, j);
            }
            if h >= 2 {
                if excess == 0 {
                    let i = r + h - 1;
                    for j in c..c + w {
                        eval_cell(i, j);
                    }
                } else {
                    let i_bot = r + h - 1;
                    let last_w = w - excess;
                    for j in c..c + last_w {
                        eval_cell(i_bot, j);
                    }
                    let i_prev = r + h - 2;
                    for j in c + last_w..c + w {
                        eval_cell(i_prev, j);
                    }
                }
            }

            for i in r + 1..r + h - 1 {
                eval_cell(i, c);
                if w >= 2 {
                    eval_cell(i, c + w - 1);
                }
            }

            (adj, adj_pond, adj_border, ef, tbonus, suff)
        };

        struct FastRect {
            r: usize,
            c: usize,
            w: usize,
            h: usize,
            excess: usize,
            comp: f64,
            adj: i32,
    adj_pond: i32,
    adj_border: i32,
            tb: f64,
            tbonus: f64,
            ef: i32,
            suff: f64,
            score: f64,
        }

        let pipe_bonus = self.calc_pipeline_bonus(dep_time);

        let get_rect_tb = |r: usize,
                           c: usize,
                           w: usize,
                           h: usize,
                           rem_time: i64,
                           _dep_time: i64,
                           n: usize,
                           global_avg_rem_time: f64|
         -> f64 {
            let cx = r as f64 + (h as f64 - 1.0) / 2.0;
            let cy = c as f64 + (w as f64 - 1.0) / 2.0;
            let dx = cx - (n as f64 - 1.0) / 2.0;
            let dy = cy - (n as f64 - 1.0) / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let norm_dist = dist / (n as f64 / 1.414);
            let mut time_factor = rem_time as f64 / f64::max(1.0, global_avg_rem_time) - 1.0;
            time_factor = f64::max(-1.0, f64::min(1.0, time_factor));
            let mut score = time_factor * norm_dist;

            let touch_top = r == 0;
            let touch_bottom = r + h == n;
            let touch_left = c == 0;
            let touch_right = c + w == n;
            let corner_anchor = (touch_top && touch_left) as i32
                + (touch_top && touch_right) as i32
                + (touch_bottom && touch_left) as i32
                + (touch_bottom && touch_right) as i32;

            if corner_anchor > 0 {
                score += 0.20 * corner_anchor as f64;
            }

            if rem_time > 10000 {
                score += 0.08 * (rem_time - 10000) as f64 / 90000.0 * norm_dist;
            }
            score
        };

        let mut min_r = self.n;
        let mut max_r = -1i32;
        let mut min_c = self.n;
        let mut max_c = -1i32;
        for i in 0..self.n {
            for j in 0..self.n {
                if self.grid[i][j] == '.' && self.own[i][j] == -1 {
                    min_r = min_r.min(i);
                    max_r = max_r.max(i as i32);
                    min_c = min_c.min(j);
                    max_c = max_c.max(j as i32);
                }
            }
        }
        let free_aspect = if max_c >= min_c as i32 && max_r >= min_r as i32 {
            (max_c - min_c as i32 + 1) as f64 / (max_r - min_r as i32 + 1) as f64
        } else {
            1.0
        };
        let is_elongated = free_aspect > 1.4 || free_aspect < 0.7;

        let elapsed_now = self.start_time.elapsed().as_secs_f64();
        let progress = (turn + 1) as f64 / m as f64;
        let zone_decay = if progress < 0.70 {
            1.0
        } else {
            f64::max(0.0, 1.0 - (progress - 0.70) / 0.30)
        };
        
        let target_time = 1.68 * progress;
        let time_margin = target_time - elapsed_now;

        let top_k = if quick_mode {
            1
        } else if elapsed_now > self.params.t_mm || time_margin < -0.15 {
            4
        } else if elapsed_now > self.params.t_deep || time_margin < -0.05 {
            8
        } else if time_margin > 0.10 {
            18
        } else {
            12
        };

        // Phase 1: 完全長方形 (w*h == p)
        {
            let t0 = Instant::now();
            let mut dims: Vec<(usize, usize)> = Vec::new();
            let mut w = 1;
            while w * w <= p {
                if p % w == 0 {
                    let h = p / w;
                    if w <= self.n && h <= self.n {
                        dims.push((w, h));
                    }
                    if w != h && h <= self.n && w <= self.n {
                        dims.push((h, w));
                    }
                }
                w += 1;
            }
            dims.sort_unstable_by(|a, b| {
                if is_elongated {
                    let aspect_a = a.0 as f64 / a.1 as f64;
                    let aspect_b = b.0 as f64 / b.1 as f64;
                    ((aspect_a / free_aspect).ln().abs())
                        .total_cmp(&(aspect_b / free_aspect).ln().abs())
                } else {
                    (a.0 as i32 - a.1 as i32)
                        .abs()
                        .cmp(&(b.0 as i32 - b.1 as i32).abs())
                }
            });

            let mut fast_rects: Vec<FastRect> = Vec::new();
            for &(w, h) in &dims {
                if self.start_time.elapsed().as_secs_f64() > 1.82 {
                    break;
                }
                let comp = 2.0 * (p as f64).sqrt() / (w + h) as f64;
                for r in 0..=self.n - h {
                    for c in 0..=self.n - w {
                        if self.count_blocked(r, c, h, w) == 0 {
                            let tb = get_rect_tb(
                                r,
                                c,
                                w,
                                h,
                                rem_time,
                                dep_time,
                                self.n,
                                self.global_avg_rem_time,
                            ) * zone_decay;
                            let (adj, adj_pond, adj_border, ef, tbonus, suff) = eval_approx_rect(
                                r, c, w, h, 0, dep_time, &self.own, &self.grid, &self.grp, self.n,
                            );
                            
                            let quick_cap = self.count_free(r.saturating_sub(1), c.saturating_sub(1), (h + 2).min(self.n - r.saturating_sub(1)), (w + 2).min(self.n - c.saturating_sub(1))) - (w * h) as i32;
                            let score = self.calc_heuristics_score(v, base_cost, comp, adj, adj_pond, adj_border, tbonus, ef, tb, quick_cap, pipe_bonus, suff, 0.0);
                            
                            fast_rects.push(FastRect {
                                r, c, w, h, excess: 0, comp, adj, adj_pond, adj_border, tb, tbonus, ef, suff, score,
                            });
                        }
                    }
                }
            }

            if !fast_rects.is_empty() {
                fast_rects.sort_unstable_by(|a, b| b.score.total_cmp(&a.score));
                if quick_mode {
                    let fr = &fast_rects[0];
                    let cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, 0);
                    let cand = PlaceResult {
                        cells,
                        compact: fr.comp,
                        adj: fr.adj,
                            adj_pond: fr.adj_pond,
                            adj_border: fr.adj_border,
                        tie_breaker: fr.tb,
                        time_bonus: fr.tbonus,
                        exposed_free: fr.ef,
                        max_free_rect: 0,
                        pipe_bonus,
                        suffocate_penalty: fr.suff,
                        pocket_fit: 0.0,
                    };
                    if is_better(&cand, &best, v, base_cost, &self.params) {
                        best = cand;
                    }
                } else {
                    let eval_limit = if elapsed_now < 1.55 { top_k.min(16) } else { top_k.min(3) };
                    if fast_rects.len() > eval_limit {
                        fast_rects.truncate(eval_limit);
                    }
                    for fr in &fast_rects {
                        let cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, 0);
                        let cap = if elapsed_now < 1.40 { self.eval_cap(&cells) } else { 0 };
                        let cand = PlaceResult {
                            cells,
                            compact: fr.comp,
                            adj: fr.adj,
                            adj_pond: fr.adj_pond,
                            adj_border: fr.adj_border,
                            tie_breaker: fr.tb,
                            time_bonus: fr.tbonus,
                            exposed_free: fr.ef,
                            max_free_rect: cap,
                            pipe_bonus,
                            suffocate_penalty: fr.suff,
                            pocket_fit: 0.0,
                        };
                        if is_better(&cand, &best, v, base_cost, &self.params) {
                            best = cand;
                        }
                    }
                }
            }
            self.diag.time_phase1_us += t0.elapsed().as_micros();
        }

        // Phase 2: 近似長方形
        {
            let t0 = Instant::now();
            if best.compact < 0.98 {
                let mut dims: Vec<(usize, usize, usize)> = Vec::new();
                let sq = 2.max((p as f64).sqrt().round() as usize);
                let max_w = self.n.min(sq + 5);
                for w in 2..=max_w {
                    let h = (p + w - 1) / w;
                    if h > self.n || h < 2 {
                        continue;
                    }
                    let excess = w * h - p;
                    if excess > 0 && excess < w {
                        dims.push((w, h, excess));
                    }
                }
                dims.sort_unstable_by(|a, b| {
                    if is_elongated {
                        let aspect_a = a.0 as f64 / a.1 as f64;
                        let aspect_b = b.0 as f64 / b.1 as f64;
                        ((aspect_a / free_aspect).ln().abs())
                            .total_cmp(&(aspect_b / free_aspect).ln().abs())
                    } else {
                        (a.0 as i32 - a.1 as i32)
                            .abs()
                            .cmp(&(b.0 as i32 - b.1 as i32).abs())
                    }
                });

                let limit_dims = if quick_mode {
                    3
                } else if time_margin > 0.10 {
                    15
                } else if time_margin > -0.05 {
                    10
                } else {
                    5
                };
                if dims.len() > limit_dims {
                    dims.truncate(limit_dims);
                }

                let mut p2_fast: Vec<FastRect> = Vec::new();
                for &(w, h, excess) in &dims {
                    if self.start_time.elapsed().as_secs_f64() > 1.82 {
                        break;
                    }
                    let last_w = w - excess;
                    for r in 0..=self.n - h {
                        for c in 0..=self.n - w {
                            if h >= 2
                                && self.count_blocked(r, c, h - 1, w) == 0
                                && self.count_blocked(r + h - 1, c, 1, last_w) == 0
                            {
                                let comp = 4.0 * (p as f64).sqrt()
                                    / (2 * (w + h) + if excess > 0 { 2 } else { 0 }) as f64;
                                let tb = get_rect_tb(
                                    r,
                                    c,
                                    w,
                                    h,
                                    rem_time,
                                    dep_time,
                                    self.n,
                                    self.global_avg_rem_time,
                                ) * zone_decay;
                                let (adj, adj_pond, adj_border, ef, tbonus, suff) = eval_approx_rect(
                                    r, c, w, h, excess, dep_time, &self.own, &self.grid, &self.grp,
                                    self.n,
                                );
                                
                                let quick_cap = self.count_free(r.saturating_sub(1), c.saturating_sub(1), (h + 2).min(self.n - r.saturating_sub(1)), (w + 2).min(self.n - c.saturating_sub(1))) - (w * h - excess) as i32;
                                let score = self.calc_heuristics_score(v, base_cost, comp, adj, adj_pond, adj_border, tbonus, ef, tb, quick_cap, pipe_bonus, suff, 0.0);
                                
                                p2_fast.push(FastRect {
                                    r, c, w, h, excess, comp, adj, adj_pond, adj_border, tb, tbonus, ef, suff, score,
                                });
                            }
                        }
                    }
                }

                if !p2_fast.is_empty() {
                    p2_fast.sort_unstable_by(|a, b| b.score.total_cmp(&a.score));
                    if quick_mode {
                        let fr = &p2_fast[0];
                        let cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, fr.excess);
                        let comp = self.compactness(&cells);
                        let cand = PlaceResult {
                            cells,
                            compact: comp,
                            adj: fr.adj,
                            adj_pond: fr.adj_pond,
                            adj_border: fr.adj_border,
                            tie_breaker: fr.tb,
                            time_bonus: fr.tbonus,
                            exposed_free: fr.ef,
                            max_free_rect: 0,
                            pipe_bonus,
                            suffocate_penalty: fr.suff,
                            pocket_fit: 0.0,
                        };
                        if is_better(&cand, &best, v, base_cost, &self.params) {
                            best = cand;
                        }
                    } else {
                        let eval_limit = if elapsed_now < 1.55 { top_k.min(6) } else { top_k.min(3) };
                        if p2_fast.len() > eval_limit {
                            p2_fast.truncate(eval_limit);
                        }
                        for fr in &p2_fast {
                            let cells = get_rect_cells(fr.r, fr.c, fr.w, fr.h, fr.excess);
                            let comp = self.compactness(&cells);
                            let cap = if elapsed_now < 1.40 { self.eval_cap(&cells) } else { 0 };
                            let cand = PlaceResult {
                                cells,
                                compact: comp,
                                adj: fr.adj,
                            adj_pond: fr.adj_pond,
                            adj_border: fr.adj_border,
                                tie_breaker: fr.tb,
                                time_bonus: fr.tbonus,
                                exposed_free: fr.ef,
                                max_free_rect: cap,
                                pipe_bonus,
                                suffocate_penalty: fr.suff,
                                pocket_fit: 0.0,
                            };
                            if is_better(&cand, &best, v, base_cost, &self.params) {
                                best = cand;
                            }
                        }
                    }
                }
            }
            self.diag.time_phase2_us += t0.elapsed().as_micros();
        }

        if quick_mode {
            if best.cells.is_empty() && self.start_time.elapsed().as_secs_f64() >= 1.82 {
                let t_fallback = Instant::now();
                let fallback_cells = self.fallback_bfs(p);
                
                if !fallback_cells.is_empty() {
                    let comp = self.compactness(&fallback_cells);
                    best = PlaceResult {
                        cells: fallback_cells,
                        compact: comp,
                        adj: 0,
                        adj_pond: 0,
                        adj_border: 0,
                        tie_breaker: 0.0,
                        time_bonus: 0.0,
                        exposed_free: 0,
                        max_free_rect: 0,
                        pipe_bonus,
                        suffocate_penalty: 0.0,
                        pocket_fit: 0.0,
                    };
                }
            }
            return best;
        }

        let p2_skip_thresh = if elapsed_now > 1.50 || time_margin < -0.10 {
            0.9
        } else {
            1.0
        };
        if (elapsed_now > 1.85 || best.compact > p2_skip_thresh) && !best.cells.is_empty() {
            return best;
        }

        // Phase 2.5: Box内BFS
        {
            let t0 = Instant::now();
            let sq = 2.max((p as f64).sqrt().round() as usize);
            let mut box_dims: Vec<(usize, usize)> = Vec::new();
            let max_bw = self.n.min(sq + 5);
            for bw in sq..=max_bw {
                let min_bh = 2.max((p + bw - 1) / bw);
                let max_bh = self.n.min(bw + 3);
                for bh in min_bh..=max_bh {
                    if bw * bh >= p {
                        box_dims.push((bw, bh));
                        if bw != bh {
                            box_dims.push((bh, bw));
                        }
                    }
                }
            }
            box_dims.sort_unstable_by(|a, b| {
                let area_a = a.0 * a.1;
                let area_b = b.0 * b.1;
                if area_a != area_b {
                    area_a.cmp(&area_b)
                } else {
                    (a.0 as i32 - a.1 as i32)
                        .abs()
                        .cmp(&(b.0 as i32 - b.1 as i32).abs())
                }
            });
            box_dims.dedup();
            let max_boxes = if time_margin < -0.05 { 2 } else { 4 };
            if box_dims.len() > max_boxes {
                box_dims.truncate(max_boxes);
            }

            let mut p25_cands: Vec<PlaceResult> = Vec::new();
            for &(bw, bh) in &box_dims {
                if bw > self.n || bh > self.n {
                    continue;
                }
                for r in 0..=self.n - bh {
                    for c in 0..=self.n - bw {
                        let free = self.count_free(r, c, bh, bw);
                        if free < p as i32 {
                            continue;
                        }
                        let reg = self.build_box_bfs(r, c, bh, bw, p);
                        if !reg.is_empty() {
                            let comp = self.compactness(&reg);
                            let (adj, adj_pond, adj_border) = self.calc_adjacency_split(&reg);
                            let tb = self.get_tie_breaker(&reg, rem_time, dep_time) * zone_decay;
                            let ef = self.calc_exposed_free(&reg);
                            let suff = self.predict_future_suffocation(&reg, dep_time);
                            let pfit = self.calc_pocket_fit(&reg);
                            p25_cands.push(PlaceResult {
                                cells: reg,
                                compact: comp,
                                adj,
                                adj_pond,
                                adj_border,
                                tie_breaker: tb,
                                time_bonus: 0.0,
                                exposed_free: ef,
                                max_free_rect: 0,
                                pipe_bonus,
                                suffocate_penalty: suff,
                                pocket_fit: pfit,
                            });
                        }
                    }
                }
            }

            if !p25_cands.is_empty() {
                p25_cands.sort_unstable_by(|a, b| {
                    let get_score = |r: &PlaceResult| -> f64 {
                        self.calc_heuristics_score(v, base_cost, r.compact, r.adj, r.adj_pond, r.adj_border, r.time_bonus, r.exposed_free, r.tie_breaker, r.max_free_rect, r.pipe_bonus, r.suffocate_penalty, r.pocket_fit)
                    };
                    get_score(b).total_cmp(&get_score(a))
                });
                
                let eval_limit = if elapsed_now < 1.55 { top_k.min(5) } else { top_k.min(2) };
                if p25_cands.len() > eval_limit {
                    p25_cands.truncate(eval_limit);
                }

                for cand in &mut p25_cands {
                    if elapsed_now < 1.55 {
                        cand.max_free_rect = self.eval_cap(&cand.cells);
                    }
                    if is_better(cand, &best, v, base_cost, &self.params) {
                        best = cand.clone();
                    }
                }
            }
            self.diag.time_phase25_us += t0.elapsed().as_micros();
        }

        let p25_skip_thresh = if elapsed_now > 1.50 || time_margin < -0.10 {
            0.85
        } else {
            0.98
        };
        if (elapsed_now > 1.85 || best.compact > p25_skip_thresh) && !best.cells.is_empty() {
            return best;
        }

        // Phase 3: ガイド付きBFS
        {
            let t0 = Instant::now();
            let mut vis = [[false; MAXN]; MAXN];
            let mut p3_cands: Vec<PlaceResult> = Vec::new();

            'p3_outer: for i in 0..self.n {
                for j in 0..self.n {
                    if self.start_time.elapsed().as_secs_f64() > 1.85 {
                        break 'p3_outer;
                    }
                    if vis[i][j] || self.grid[i][j] != '.' || self.own[i][j] != -1 {
                        continue;
                    }

                    let mut q = VecDeque::new();
                    q.push_back((i, j));
                    vis[i][j] = true;
                    let mut comp = Vec::new();

                    while let Some((x, y)) = q.pop_front() {
                        comp.push((x, y));
                        for d in 0..4 {
                            let nx = x as i32 + DR[d];
                            let ny = y as i32 + DC[d];
                            if nx >= 0 && nx < self.n as i32 && ny >= 0 && ny < self.n as i32 {
                                let nxu = nx as usize;
                                let nyu = ny as usize;
                                if !vis[nxu][nyu]
                                    && self.grid[nxu][nyu] == '.'
                                    && self.own[nxu][nyu] == -1
                                {
                                    vis[nxu][nyu] = true;
                                    q.push_back((nxu, nyu));
                                }
                            }
                        }
                    }

                    if comp.len() < p {
                        continue;
                    }

                    let mut cx = 0.0;
                    let mut cy = 0.0;
                    for &(x, y) in &comp {
                        cx += x as f64;
                        cy += y as f64;
                    }
                    cx /= comp.len() as f64;
                    cy /= comp.len() as f64;

                    struct CompCandidate {
                        pt: (usize, usize),
                        score: f64,
                    }
                    let mut scored_comp: Vec<CompCandidate> = comp
                        .iter()
                        .map(|&(x, y)| {
                            let mut adj = 0;
                            let mut pocket = 0;
                            for d in 0..4 {
                                let nx = x as i32 + DR[d];
                                let ny = y as i32 + DC[d];
                                if nx < 0
                                    || nx >= self.n as i32
                                    || ny < 0
                                    || ny >= self.n as i32
                                    || self.grid[nx as usize][ny as usize] == '#'
                                {
                                    pocket += 1;
                                    adj += 1;
                                } else if self.own[nx as usize][ny as usize] >= 0 {
                                    adj += 1;
                                }
                            }
                            let score = (x as f64 - cx) * (x as f64 - cx)
                                + (y as f64 - cy) * (y as f64 - cy)
                                - adj as f64 * 40.0
                                - pocket as f64 * 20.0;
                            CompCandidate { pt: (x, y), score }
                        })
                        .collect();

                    scored_comp.sort_unstable_by(|a, b| a.score.total_cmp(&b.score));

                    let elapsed_now = self.start_time.elapsed().as_secs_f64();
                    
                    let max_t = if elapsed_now > 1.80 {
                        if best.cells.is_empty() { 3 } else { 1 }
                    } else if elapsed_now > self.params.t_mm || time_margin < -0.15 {
                        (self.params.max_t_base / 6.0) as usize
                    } else if elapsed_now > self.params.t_deep || time_margin < -0.05 {
                        (self.params.max_t_base / 2.0) as usize
                    } else if time_margin > 0.10 {
                        (self.params.max_t_base * 1.33) as usize
                    } else {
                        self.params.max_t_base as usize
                    };
                    
                    let trials = scored_comp.len().min(max_t);
                    let mut chosen_starts: Vec<(usize, usize)> = Vec::new();
                    for t in 0..trials {
                        if self.start_time.elapsed().as_secs_f64() > 1.85 {
                            break 'p3_outer;
                        }
                        let (sx, sy) = scored_comp[t].pt;
                        let mut too_close = false;
                        for &(cx, cy) in &chosen_starts {
                            let dist_sq = (sx as i32 - cx as i32).pow(2) + (sy as i32 - cy as i32).pow(2);
                            if dist_sq <= 2 {
                                too_close = true;
                                break;
                            }
                        }
                        if too_close && chosen_starts.len() >= 10 {
                            continue;
                        }
                        chosen_starts.push((sx, sy));

                        // Phase 3a: Strict Bounding Box Dijkstra
                        let reg_strict = self.build_dijkstra_blob_strict(sx, sy, p);
                        let reg = if !reg_strict.is_empty() {
                            reg_strict
                        } else {
                            // Phase 3b: Fallback Dijkstra
                            self.build_dijkstra_blob(sx, sy, p)
                        };

                        if !reg.is_empty() {
                            let comp_val = self.compactness(&reg);
                            let (adj, adj_pond, adj_border) = self.calc_adjacency_split(&reg);
                            let tb = self.get_tie_breaker(&reg, rem_time, dep_time) * zone_decay;
                            let tbonus = self.calc_time_bonus(&reg, dep_time);
                            let ef = self.calc_exposed_free(&reg);
                            let suff = self.predict_future_suffocation(&reg, dep_time);
                            let pfit = self.calc_pocket_fit(&reg);
                            p3_cands.push(PlaceResult {
                                cells: reg,
                                compact: comp_val,
                                adj,
                                adj_pond,
                                adj_border,
                                tie_breaker: tb,
                                time_bonus: tbonus,
                                exposed_free: ef,
                                max_free_rect: 0,
                                pipe_bonus,
                                suffocate_penalty: suff,
                                pocket_fit: pfit,
                            });
                        }
                    }
                }
            }

            if !p3_cands.is_empty() {
                p3_cands.sort_unstable_by(|a, b| {
                    let get_score = |r: &PlaceResult| -> f64 {
                        self.calc_heuristics_score(v, base_cost, r.compact, r.adj, r.adj_pond, r.adj_border, r.time_bonus, r.exposed_free, r.tie_breaker, r.max_free_rect, r.pipe_bonus, r.suffocate_penalty, r.pocket_fit)
                    };
                    get_score(b).total_cmp(&get_score(a))
                });
                
                let eval_limit = if elapsed_now < 1.55 { top_k.min(5) } else { top_k.min(2) };
                if p3_cands.len() > eval_limit {
                    p3_cands.truncate(eval_limit);
                }

                for cand in &mut p3_cands {
                    if elapsed_now < 1.55 {
                        cand.max_free_rect = self.eval_cap(&cand.cells);
                    }
                    if is_better(cand, &best, v, base_cost, &self.params) {
                        best = cand.clone();
                    }
                }
            }
            self.diag.time_phase3_us += t0.elapsed().as_micros();
        }

        best
    }

    fn fallback_bfs(&mut self, p: usize) -> Vec<(usize, usize)> {
        self.bfs_stamp = self.bfs_stamp.wrapping_add(1);
        if self.bfs_stamp == 0 {
            self.vis_bfs_stamp = [[0; MAXN]; MAXN];
            self.bfs_stamp = 1;
        }
        let current_stamp = self.bfs_stamp;

        for i in 0..self.n {
            for j in 0..self.n {
                if self.grid[i][j] == '.' && self.own[i][j] == -1 && self.vis_bfs_stamp[i][j] != current_stamp {
                    let mut reg = Vec::with_capacity(p);
                    let mut q = VecDeque::new();
                    
                    q.push_back((i, j));
                    self.vis_bfs_stamp[i][j] = current_stamp;
                    
                    while let Some((x, y)) = q.pop_front() {
                        reg.push((x, y));
                        if reg.len() == p {
                            return reg;
                        }
                        
                        for d in 0..4 {
                            let nx = x as i32 + DR[d];
                            let ny = y as i32 + DC[d];
                            if nx >= 0 && nx < self.n as i32 && ny >= 0 && ny < self.n as i32 {
                                let nxu = nx as usize;
                                let nyu = ny as usize;
                                if self.grid[nxu][nyu] == '.' && self.own[nxu][nyu] == -1 && self.vis_bfs_stamp[nxu][nyu] != current_stamp {
                                    self.vis_bfs_stamp[nxu][nyu] = current_stamp;
                                    q.push_back((nxu, nyu));
                                }
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

fn is_better(a: &PlaceResult, b: &PlaceResult, v: i64, base_cost: f64, params: &Params) -> bool {
    if a.cells.is_empty() {
        return false;
    }
    if b.cells.is_empty() {
        return true;
    }

    let get_score = |r: &PlaceResult| -> f64 {
        let heuristics = params.w_adj * r.adj as f64
            + params.w_time * r.time_bonus
            - params.w_expose * r.exposed_free as f64
            + params.w_zone * r.tie_breaker
            + params.w_capacity * r.max_free_rect as f64
            + params.w_pipe * r.pipe_bonus
            - params.w_suffocate * r.suffocate_penalty
            + params.w_pocket * r.pocket_fit;
            
        (v as f64 * r.compact) + (base_cost * heuristics)
    };

    let score_a = get_score(a);
    let score_b = get_score(b);

    if (score_a - score_b).abs() < 1.0 {
        if a.adj != b.adj {
            return a.adj > b.adj;
        }
        return a.tie_breaker > b.tie_breaker;
    }
    score_a > score_b
}

struct Scanner {
    buffer: Vec<String>,
    char_buffer: VecDeque<char>,
}

impl Scanner {
    fn new() -> Self {
        Scanner {
            buffer: Vec::new(),
            char_buffer: VecDeque::new(),
        }
    }

    fn next<T: std::str::FromStr>(&mut self) -> Option<T> {
        self.char_buffer.clear();
        while self.buffer.is_empty() {
            let mut input = String::new();
            let bytes_read = io::stdin().read_line(&mut input).ok()?;
            if bytes_read == 0 {
                return None;
            }
            self.buffer = input
                .split_whitespace()
                .map(|s| s.to_string())
                .rev()
                .collect();
        }
        self.buffer.pop()?.parse::<T>().ok()
    }

    fn next_char(&mut self) -> Option<char> {
        while self.char_buffer.is_empty() {
            if let Some(token) = self.next::<String>() {
                for c in token.chars() {
                    self.char_buffer.push_back(c);
                }
            } else {
                return None;
            }
        }
        self.char_buffer.pop_front()
    }
}

fn main() {
    let start_time = Instant::now();
    
    let args: Vec<String> = std::env::args().collect();
    let params = Params {
        w_adj: args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.000930648811439626),
        w_time: args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0015812296573779147),
        w_expose: args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.004643700270792169),
        w_zone: args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0008822059737903149),
        w_capacity: args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.0019684143784392803),
        w_pipe: args.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.0010232407503122955),
        w_suffocate: args.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.003947946318465455),
        w_pocket: args.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.0006210630252290683),
        base_cost_weight: args.get(9).and_then(|s| s.parse().ok()).unwrap_or(1.8232673356521643),
        t_mm: args.get(10).and_then(|s| s.parse().ok()).unwrap_or(1.5458212320758866),
        t_deep: args.get(11).and_then(|s| s.parse().ok()).unwrap_or(1.4074403758571954),
        max_t_base: args.get(12).and_then(|s| s.parse().ok()).unwrap_or(514.1966045939967),
        
        mm_compact_th1: args.get(13).and_then(|s| s.parse().ok()).unwrap_or(0.85),
        mm_v_th: args.get(14).and_then(|s| s.parse().ok()).unwrap_or(300_000.0),
        mm_compact_th2: args.get(15).and_then(|s| s.parse().ok()).unwrap_or(0.90),
        mm_density_th: args.get(16).and_then(|s| s.parse().ok()).unwrap_or(1.5),
        mm_density_bonus_th: args.get(17).and_then(|s| s.parse().ok()).unwrap_or(2.0),
        mm_v_bonus_th: args.get(18).and_then(|s| s.parse().ok()).unwrap_or(1_000_000.0),
        mm_inter_bonus1: args.get(19).and_then(|s| s.parse().ok()).unwrap_or(3),
        mm_inter_bonus2: args.get(20).and_then(|s| s.parse().ok()).unwrap_or(4),
        density_reject_mult: args.get(21).and_then(|s| s.parse().ok()).unwrap_or(1.04),
        p3_delta_weight: args.get(22).and_then(|s| s.parse().ok()).unwrap_or(7808.192650074299),
        p3_bbox_area_mult: args.get(23).and_then(|s| s.parse().ok()).unwrap_or(1.6713648933636818),
        p3_bbox_aspect_mult: args.get(24).and_then(|s| s.parse().ok()).unwrap_or(0.6955034735504015),
        p3_lookahead_steps: args.get(25).and_then(|s| s.parse().ok()).unwrap_or(3),
        w_adj_pond: args.get(28).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        w_adj_border: args.get(29).and_then(|s| s.parse().ok()).unwrap_or(0.05),
    };

    let mut scanner = Scanner::new();
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let n: usize = match scanner.next() {
        Some(val) => val,
        None => return,
    };
    let m: usize = match scanner.next() {
        Some(val) => val,
        None => return,
    };
    let r: f64 = match scanner.next() {
        Some(val) => val,
        None => return,
    };

    let mut grid = [['.'; MAXN]; MAXN];
    let mut total_grass = 0;

    for i in 0..n {
        for j in 0..n {
            let ch: char = scanner.next_char().unwrap();
            grid[i][j] = ch;
            if ch == '.' {
                total_grass += 1;
            }
        }
    }

    let mut solver = Solver::new(n, r, grid, total_grass, start_time, params.clone());
    solver.init_ponds();

    let mut inter_mask = [false; 1000];

    for turn in 0..m {
        let _id: usize = match scanner.next() {
            Some(val) => val,
            None => break,
        };
        let s: i64 = scanner.next().unwrap();
        let t: i64 = scanner.next().unwrap();
        let p: usize = scanner.next().unwrap();
        let v: i64 = scanner.next().unwrap();

        solver.grp[turn].p = p;
        solver.grp[turn].s = s;
        solver.grp[turn].t = t;
        solver.grp[turn].v = v;
        solver.grp[turn].c_min = 0.0;
        solver.grp[turn].active = false;

        let now = solver.grp[turn].s;
        let rem_time = solver.grp[turn].t - now;

        solver.global_total_rem_time += rem_time;
        solver.global_groups_seen += 1;
        solver.global_avg_rem_time =
            solver.global_total_rem_time as f64 / solver.global_groups_seen as f64;

        let density = v as f64 / (p as f64 * rem_time as f64);
        solver.global_total_density += density;
        solver.global_density_count += 1;
        solver.global_avg_density =
            solver.global_total_density / solver.global_density_count as f64;

        for j in 0..turn {
            if solver.grp[j].active && solver.grp[j].t < now {
                let fee = (solver.grp[j].v as f64 * solver.grp[j].c_min + 0.5).floor() as i64;
                solver.diag.total_fee += fee;
                solver.diag.bucket_fee[size_bucket(solver.grp[j].p)] += fee;

                match solver.grp[j].phase_id {
                    1 => solver.diag.phase1_fee += fee,
                    2 => solver.diag.phase2_fee += fee,
                    25 => solver.diag.phase25_fee += fee,
                    _ => solver.diag.phase3_fee += fee,
                }

                solver.clear_grp_cells(j);
                solver.grp[j].cells.clear();
                solver.grp[j].active = false;
            }
        }

        let mut cur_free = 0;
        for i in 0..n {
            for j in 0..n {
                if solver.grid[i][j] == '.' && solver.own[i][j] == -1 {
                    cur_free += 1;
                }
            }
        }
        let free_rate = cur_free as f64 / solver.total_grass as f64;

        let occ = 1.0 - free_rate;
        let bucket = (turn * 10 / m).min(9);
        solver.diag.occupancy_history[bucket] += occ;

        let p_factor = 1.20 - 0.01 * (p as f64 / 150.0);
        let current_thresh = if turn >= m - 40 {
            0.0
        } else {
            (0.24 + 0.60 * (1.0 - free_rate)) * p_factor * solver.params.density_reject_mult
        };

        if solver.global_density_count > 20
            && density < solver.global_avg_density * current_thresh
        {
            writeln!(out, "0\nNo").unwrap();
            out.flush().unwrap();
            solver.diag.rejected += 1;
            solver.diag.bucket_reject[size_bucket(p)] += 1;
            continue;
        }

        let is_hard_mode = solver.start_time.elapsed().as_secs_f64() >= 1.82;
        let result = solver.find_best_placement(p, rem_time, solver.grp[turn].t, v, is_hard_mode, turn, m);
        
        // Optunaパラメータを使ってハイブリッドにMultiMoveの発動を判定
        let try_multimove = result.cells.is_empty()
            || (result.compact < solver.params.mm_compact_th1 && (v as f64) > solver.params.mm_v_th)
            || (result.compact < solver.params.mm_compact_th2 && density > solver.global_avg_density * solver.params.mm_density_th);

        if !try_multimove {
            writeln!(out, "0\nYes").unwrap();
            for &(x, y) in &result.cells {
                writeln!(out, "{} {}", x, y).unwrap();
            }
            solver.set_cells(&result.cells, turn as i32);
            solver.grp[turn].cells = result.cells.clone();
            solver.grp[turn].c_min = result.compact;
            solver.grp[turn].active = true;

            solver.diag.accepted += 1;
            solver.diag.total_compact += result.compact;
            let b = size_bucket(p);
            solver.diag.bucket_n[b] += 1;
            solver.diag.bucket_compact_sum[b] += result.compact;

            if result.compact > 0.95 {
                solver.grp[turn].phase_id = 1; solver.diag.phase1_compact += solver.grp[turn].c_min;
                solver.diag.phase1_count += 1;
            } else if result.compact > 0.85 {
                solver.grp[turn].phase_id = 2; solver.diag.phase2_compact += solver.grp[turn].c_min;
                solver.diag.phase2_count += 1;
            } else if result.compact > 0.75 {
                solver.grp[turn].phase_id = 25; solver.diag.phase25_compact += solver.grp[turn].c_min;
                solver.diag.phase25_count += 1;
            } else {
                solver.grp[turn].phase_id = 3; solver.diag.phase3_compact += solver.grp[turn].c_min;
                solver.diag.phase3_count += 1;
            }
        } else {
            let t0_mm = Instant::now();
            let mut best_mm = MultiMovePlan {
                icells: Vec::new(),
                net_gain: if !result.cells.is_empty() {
                    (v as f64 * result.compact + 0.5).floor() as i64
                } else {
                    0
                },
                intersecting: Vec::new(),
                move_plans: Vec::new(),
            };

            let elapsed = solver.start_time.elapsed().as_secs_f64();
            
            if elapsed < solver.params.t_mm {
                'mm_end: {
                    let mut box_shapes: Vec<(usize, usize)> = Vec::new();
                    for w in 1..=n {
                        let h = (p + w - 1) / w;
                        if h >= 1 && h <= n {
                            let excess = h * w - p;
                            if excess < w
                                && (h.max(w) as f64 / h.min(w) as f64) <= 1.8
                            {
                                box_shapes.push((h, w));
                            }
                        }
                    }

                    box_shapes.sort_unstable_by(|a, b| {
                        (a.0 as i32 - a.1 as i32)
                            .abs()
                            .cmp(&(b.0 as i32 - b.1 as i32).abs())
                    });

                    if box_shapes.len() > 10 {
                        box_shapes.truncate(10);
                    }

                    for &(h, w) in &box_shapes {
                        let excess = h * w - p;
                        let mut icells = Vec::new();
                        for i in 0..h - 1 {
                            for j in 0..w {
                                icells.push((i, j));
                            }
                        }
                        for j in 0..w - excess {
                            icells.push((h - 1, j));
                        }
                        let box_comp = solver.compactness(&icells);
                        let base_expected_fee = (v as f64 * box_comp + 0.5).floor() as i64;

                        for r in 0..=n - h {
                            for c in 0..=n - w {
                                if solver.start_time.elapsed().as_secs_f64() > solver.params.t_mm {
                                    break 'mm_end;
                                }

                                let ponds = solver.count_ponds(r, c, h - 1, w)
                                    + solver.count_ponds(r + h - 1, c, 1, w - excess);
                                if ponds > 0 {
                                    continue;
                                }

                                let occ = (solver.count_blocked(r, c, h - 1, w)
                                    - solver.count_ponds(r, c, h - 1, w))
                                    + (solver.count_blocked(r + h - 1, c, 1, w - excess)
                                        - solver.count_ponds(r + h - 1, c, 1, w - excess));
                                if occ == 0 || occ > 60 {
                                    continue;
                                }

                                let mut intersecting: Vec<usize> = Vec::new();
                                let mut ok = true;
                                let mut running_move_cost: i64 = 0;

                                let mut max_inter = if solver.r <= 0.005 {
                                    if elapsed < 1.20 { 10 } else { 6 }
                                } else if solver.r <= 0.01 {
                                    if elapsed < 1.20 { 8 } else { 5 }
                                } else if solver.r <= 0.02 {
                                    5
                                } else {
                                    3
                                };

                                // パラメータを使って「コスパ」や「絶対価値」に応じた追加人数のボーナスを適用
                                if density > solver.global_avg_density * solver.params.mm_density_bonus_th {
                                    max_inter += solver.params.mm_inter_bonus1;
                                }
                                if (v as f64) > solver.params.mm_v_bonus_th || p >= 50 {
                                    max_inter += solver.params.mm_inter_bonus2;
                                }
                                
                                if elapsed > 1.60 {
                                    max_inter = max_inter.min(4);
                                }

                                let cost_ratio = 0.35 + 0.50 * (1.0f64).min(v as f64 / 1_000_000.0);
                                let max_allowed_move_cost = (base_expected_fee as f64 * cost_ratio) as i64;

                                for i in r..r + h {
                                    if !ok {
                                        break;
                                    }
                                    for j in c..c + w {
                                        if i == r + h - 1 && j >= c + w - excess {
                                            continue;
                                        }
                                        if solver.own[i][j] != -1 {
                                            let id = solver.own[i][j] as usize;
                                            if !inter_mask[id] {
                                                inter_mask[id] = true;
                                                intersecting.push(id);
                                                
                                                let mc = 1i64.max(
                                                    (solver.grp[id].v as f64 * solver.r + 0.5).floor() as i64,
                                                );
                                                running_move_cost += mc;

                                                if intersecting.len() > max_inter 
                                                    || running_move_cost > max_allowed_move_cost 
                                                    || (base_expected_fee - running_move_cost) <= best_mm.net_gain 
                                                {
                                                    ok = false;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                for &id in &intersecting {
                                    inter_mask[id] = false;
                                }
                                if !ok || intersecting.is_empty() {
                                    continue;
                                }

                                icells.clear();
                                for i in 0..h - 1 {
                                    for j in 0..w {
                                        icells.push((r + i, c + j));
                                    }
                                }
                                for j in 0..w - excess {
                                    icells.push((r + h - 1, c + j));
                                }
                                let actual_box_comp = solver.compactness(&icells);
                                let expected_fee =
                                    (v as f64 * actual_box_comp + 0.5).floor() as i64;

                                for &id in &intersecting {
                                    solver.clear_grp_cells(id);
                                }
                                for &(cx, cy) in &icells {
                                    solver.own[cx][cy] = -2;
                                }

                                let mut inter_sorted = intersecting.clone();
                                inter_sorted.sort_unstable_by(|&a, &b| {
                                    solver.grp[b].p.cmp(&solver.grp[a].p)
                                });

                                let sum_inter_p: usize = inter_sorted.iter().map(|&id| solver.grp[id].p).sum();
                                let free_count = cur_free + sum_inter_p - p;
                                
                                let max_p = solver.grp[inter_sorted[0]].p;

                                let mut actual_loss: i64 = 0;
                                let mut possible = free_count >= max_p;
                                let mut move_plans: Vec<(usize, Vec<(usize, usize)>)> = Vec::new();

                                if possible {
                                    for &id in &inter_sorted {
                                        if solver.start_time.elapsed().as_secs_f64() > solver.params.t_mm {
                                            for &(cx, cy) in &icells {
                                                solver.own[cx][cy] = -1;
                                            }
                                            for &tid in &intersecting {
                                                solver.set_grp_cells(tid, tid as i32);
                                            }
                                            solver.compute_bps();
                                            break 'mm_end;
                                        }

                                        let res = solver.find_best_placement(
                                            solver.grp[id].p,
                                            solver.grp[id].t - now,
                                            solver.grp[id].t,
                                            solver.grp[id].v,
                                            true,
                                            turn,
                                            m,
                                        );
                                        if res.cells.is_empty() {
                                            possible = false;
                                            break;
                                        }
                                        solver.set_cells(&res.cells, id as i32);
                                        move_plans.push((id, res.cells.clone()));
                                        let old_fee = (solver.grp[id].v as f64 * solver.grp[id].c_min + 0.5).floor() as i64;
                                        let new_fee = (solver.grp[id].v as f64 * solver.grp[id].c_min.min(res.compact) + 0.5).floor() as i64;
                                        actual_loss += old_fee - new_fee;
                                    }
                                }

                                for plan in &move_plans {
                                    solver.clear_cells(&plan.1);
                                }
                                for &(cx, cy) in &icells {
                                    solver.own[cx][cy] = -1;
                                }
                                for &id in &intersecting {
                                    solver.set_grp_cells(id, id as i32);
                                }
                                solver.compute_bps();

                                if possible {
                                    let net_gain = expected_fee - running_move_cost - actual_loss;
                                    if net_gain > best_mm.net_gain {
                                        best_mm.net_gain = net_gain;
                                        best_mm.icells = icells.clone();
                                        best_mm.intersecting = intersecting;
                                        best_mm.move_plans = move_plans;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            solver.diag.time_multimove_us += t0_mm.elapsed().as_micros();

            if best_mm.net_gain > 0 && !best_mm.icells.is_empty() {
                writeln!(out, "{}", best_mm.move_plans.len()).unwrap();

                for plan in &best_mm.move_plans {
                    let id = plan.0;
                    solver.clear_grp_cells(id);
                }

                for plan in &best_mm.move_plans {
                    let id = plan.0;
                    let mc = 1i64.max((solver.grp[id].v as f64 * solver.r + 0.5).floor() as i64);
                    let new_comp = solver.compactness(&plan.1);

                    writeln!(out, "{}", id).unwrap();
                    for &(x, y) in &plan.1 {
                        writeln!(out, "{} {}", x, y).unwrap();
                    }
                    solver.diag.total_move_cost += mc;
                    solver.diag.move_count += 1;

                    solver.grp[id].cells = plan.1.clone();
                    solver.set_cells(&plan.1, id as i32);
                    solver.grp[id].c_min = solver.grp[id].c_min.min(new_comp);
                }

                writeln!(out, "Yes").unwrap();
                for &(x, y) in &best_mm.icells {
                    writeln!(out, "{} {}", x, y).unwrap();
                }
                solver.set_cells(&best_mm.icells, turn as i32);
                solver.grp[turn].cells = best_mm.icells.clone();
                solver.grp[turn].c_min = solver.compactness(&best_mm.icells);
                solver.grp[turn].active = true;
                solver.grp[turn].phase_id = 3; solver.diag.phase3_compact += solver.grp[turn].c_min;

                solver.diag.accepted += 1;
                solver.diag.total_compact += solver.grp[turn].c_min;
                let b = size_bucket(p);
                solver.diag.bucket_n[b] += 1;
                solver.diag.bucket_compact_sum[b] += solver.grp[turn].c_min;
                solver.diag.phase3_count += 1;
            } else if !result.cells.is_empty() {
                writeln!(out, "0\nYes").unwrap();
                for &(x, y) in &result.cells {
                    writeln!(out, "{} {}", x, y).unwrap();
                }
                solver.set_cells(&result.cells, turn as i32);
                solver.grp[turn].cells = result.cells.clone();
                solver.grp[turn].c_min = result.compact;
                solver.grp[turn].active = true;

                solver.diag.accepted += 1;
                solver.diag.total_compact += result.compact;
                let b = size_bucket(p);
                solver.diag.bucket_n[b] += 1;
                solver.diag.bucket_compact_sum[b] += result.compact;

                if result.compact > 0.95 {
                    solver.grp[turn].phase_id = 1; solver.diag.phase1_compact += solver.grp[turn].c_min;
                    solver.diag.phase1_count += 1;
                } else if result.compact > 0.85 {
                    solver.grp[turn].phase_id = 2; solver.diag.phase2_compact += solver.grp[turn].c_min;
                    solver.diag.phase2_count += 1;
                } else if result.compact > 0.75 {
                    solver.grp[turn].phase_id = 25; solver.diag.phase25_compact += solver.grp[turn].c_min;
                    solver.diag.phase25_count += 1;
                } else {
                    solver.grp[turn].phase_id = 3; solver.diag.phase3_compact += solver.grp[turn].c_min;
                    solver.diag.phase3_count += 1;
                }
            } else {
                writeln!(out, "0\nNo").unwrap();
                solver.diag.rejected += 1;
                solver.diag.bucket_reject[size_bucket(p)] += 1;
            }
        }
        out.flush().unwrap();
    }

    for j in 0..m {
        if solver.grp[j].active {
            let fee = (solver.grp[j].v as f64 * solver.grp[j].c_min + 0.5).floor() as i64;
            solver.diag.total_fee += fee;
            solver.diag.bucket_fee[size_bucket(solver.grp[j].p)] += fee;

            match solver.grp[j].phase_id {
                1 => solver.diag.phase1_fee += fee,
                2 => solver.diag.phase2_fee += fee,
                25 => solver.diag.phase25_fee += fee,
                _ => solver.diag.phase3_fee += fee,
            }
        }
    }

    let elapsed = solver.start_time.elapsed().as_secs_f64();
    let bnames = ["P=4-30", "P=31-70", "P=71-110", "P=111-150"];

    let total_us = (elapsed * 1_000_000.0) as f64;
    let p1_ms = solver.diag.time_phase1_us as f64 / 1000.0;
    let p2_ms = solver.diag.time_phase2_us as f64 / 1000.0;
    let p25_ms = solver.diag.time_phase25_us as f64 / 1000.0;
    let p3_ms = solver.diag.time_phase3_us as f64 / 1000.0;
    let cap_ms = solver.diag.time_eval_cap_us as f64 / 1000.0;
    let mm_ms = solver.diag.time_multimove_us as f64 / 1000.0;

    eprintln!("\n==============================================");
    eprintln!("   AHC069 High-Performance Diagnostics Report");
    eprintln!("==============================================");
    eprintln!("Execution Time: {:.3} s / Limit: 2.000s", elapsed);
    if elapsed >= 1.82 {
        eprintln!("[WARNING] High Execution Time (>1.82s)! Time Safety Guard Engaged.");
    } else {
        eprintln!("[STATUS] Execution Time SAFE (<1.82s).");
    }
    eprintln!(
        "Map Grass: {}/{} ({:.1}%) | R = {:.4}",
        solver.total_grass,
        solver.n * solver.n,
        100.0 * solver.total_grass as f64 / (solver.n * solver.n) as f64,
        solver.r
    );
    eprintln!("--- Detailed Profiling Breakdown ---");
    eprintln!("  Phase 1 (Exact Rect)   : {:8.2} ms ({:.1}%)", p1_ms, p1_ms * 1000.0 / total_us * 100.0);
    eprintln!("  Phase 2 (Approx Rect)  : {:8.2} ms ({:.1}%)", p2_ms, p2_ms * 1000.0 / total_us * 100.0);
    eprintln!("  Phase 2.5 (Box BFS)    : {:8.2} ms ({:.1}%)", p25_ms, p25_ms * 1000.0 / total_us * 100.0);
    eprintln!("  Phase 3 (Dijkstra)     : {:8.2} ms ({:.1}%)", p3_ms, p3_ms * 1000.0 / total_us * 100.0);
    eprintln!("  Capacity Eval (Cap DP) : {:8.2} ms ({:.1}%)", cap_ms, cap_ms * 1000.0 / total_us * 100.0);
    eprintln!("  MultiMove Search       : {:8.2} ms ({:.1}%)", mm_ms, mm_ms * 1000.0 / total_us * 100.0);

    let total_req = solver.diag.accepted + solver.diag.rejected;
    let rate = 100.0 * solver.diag.accepted as f64 / 1.max(total_req) as f64;
    eprintln!(
        "Acceptance: Accepted={} / Rejected={} (Rate: {:.1}%)",
        solver.diag.accepted, solver.diag.rejected, rate
    );
    if solver.diag.accepted > 0 {
        eprintln!(
            "Average Compactness: {:.4}",
            solver.diag.total_compact / solver.diag.accepted as f64
        );
    }
    eprintln!(
        "Phase Distribution: Phase1={} | Phase2={} | Phase2.5={} | Phase3={}",
        solver.diag.phase1_count,
        solver.diag.phase2_count,
        solver.diag.phase25_count,
        solver.diag.phase3_count
    );

    let total_fee_safe = 1.max(solver.diag.total_fee) as f64;
    eprintln!("--- Per Phase Score (Fee) Breakdown ---");
    eprintln!(
        "  Phase 1 (C > 0.95)   : count={:3} | avgC={:.3} | fee={:10} ({:5.1}%)",
        solver.diag.phase1_count,
        solver.diag.phase1_compact / f64::max(1.0, solver.diag.phase1_count as f64),
        solver.diag.phase1_fee,
        solver.diag.phase1_fee as f64 / total_fee_safe * 100.0
    );
    eprintln!(
        "  Phase 2 (C > 0.85)   : count={:3} | avgC={:.3} | fee={:10} ({:5.1}%)",
        solver.diag.phase2_count,
        solver.diag.phase2_compact / f64::max(1.0, solver.diag.phase2_count as f64),
        solver.diag.phase2_fee,
        solver.diag.phase2_fee as f64 / total_fee_safe * 100.0
    );
    eprintln!(
        "  Phase 2.5 (C > 0.75) : count={:3} | avgC={:.3} | fee={:10} ({:5.1}%)",
        solver.diag.phase25_count,
        solver.diag.phase25_compact / f64::max(1.0, solver.diag.phase25_count as f64),
        solver.diag.phase25_fee,
        solver.diag.phase25_fee as f64 / total_fee_safe * 100.0
    );
    eprintln!(
        "  Phase 3 / MultiMove  : count={:3} | avgC={:.3} | fee={:10} ({:5.1}%)",
        solver.diag.phase3_count,
        solver.diag.phase3_compact / f64::max(1.0, solver.diag.phase3_count as f64),
        solver.diag.phase3_fee,
        solver.diag.phase3_fee as f64 / total_fee_safe * 100.0
    );

    eprintln!(
        "Moves: {} (Cost: {})",
        solver.diag.move_count, solver.diag.total_move_cost
    );
    eprintln!(
        "Net Fee: {} - {} = {}",
        solver.diag.total_fee,
        solver.diag.total_move_cost,
        solver.diag.total_fee - solver.diag.total_move_cost
    );
    eprintln!("--- Per size bucket ---");
    for b in 0..4 {
        eprint!(
            "  {}: acc={} rej={}",
            bnames[b], solver.diag.bucket_n[b], solver.diag.bucket_reject[b]
        );
        if solver.diag.bucket_n[b] > 0 {
            eprint!(
                " avgC={:.4}",
                solver.diag.bucket_compact_sum[b] / solver.diag.bucket_n[b] as f64
            );
        }
        eprintln!(" fee={}", solver.diag.bucket_fee[b]);
    };
    eprint!("Occupancy Transition (%):");
    for i in 0..10 {
        let bucket_size = (m / 10).max(1) as f64;
        eprint!(" {:.2}", solver.diag.occupancy_history[i] / bucket_size * 100.0);
    }
    eprintln!();

    // Optunaが読み取れるように、最終スコアを標準エラー出力へ表示
    eprintln!("SCORE: {}", solver.diag.total_fee - solver.diag.total_move_cost);
}

// fee 6919, score 6906