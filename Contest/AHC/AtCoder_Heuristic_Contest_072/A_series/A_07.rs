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

// 固定長配列による超高速・ゼロアロケーション盤面（サイズ 3.6KB、L1キャッシュに完全常駐）
#[derive(Clone, Copy)]
struct StackBoard {
    stacks: [[usize; 8]; 400],
    lens: [usize; 400],
}

impl StackBoard {
    fn new(n: usize, init_stacks: &[Vec<usize>]) -> Self {
        let mut board = Self {
            stacks: [[0; 8]; 400],
            lens: [0; 400],
        };
        for p in 0..n * n {
            let len = init_stacks[p].len();
            board.lens[p] = len;
            for i in 0..len {
                board.stacks[p][i] = init_stacks[p][i];
            }
        }
        board
    }

    #[inline(always)]
    fn len(&self, p: usize) -> usize {
        self.lens[p]
    }

    #[inline(always)]
    fn is_empty(&self, p: usize) -> bool {
        self.lens[p] == 0
    }

    #[inline(always)]
    fn last(&self, p: usize) -> Option<usize> {
        if self.lens[p] == 0 {
            None
        } else {
            Some(self.stacks[p][self.lens[p] - 1])
        }
    }

    #[inline(always)]
    fn pop(&mut self, p: usize) -> Option<usize> {
        if self.lens[p] == 0 {
            None
        } else {
            self.lens[p] -= 1;
            Some(self.stacks[p][self.lens[p]])
        }
    }

    #[inline(always)]
    fn contains(&self, p: usize, color: usize) -> bool {
        for i in 0..self.lens[p] {
            if self.stacks[p][i] == color {
                return true;
            }
        }
        false
    }
}

// 世代カウンタによる配列ゼロクリア不要の高速BFSバッファ
struct BfsContext {
    token: usize,
    visited: [usize; 400],
    dist: [usize; 400],
    prev: [usize; 400],
    prev_action: [(usize, usize, usize); 400], // (k, d, l)
    queue: [usize; 400],
    actions: [(usize, usize, usize, usize); 400], // (from_p, k, d, l)

    token_tower: usize,
    visited_tower: [usize; 3600],
    dist_tower: [usize; 3600],
    prev_tower: [(usize, usize); 3600], // (prev_idx, dir)
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

// トランポリン跳躍を活用した1匹移動のゼロアロケーションBFS
fn find_path_single_trampoline_fast(
    n: usize,
    floor: &[bool],
    board: &StackBoard,
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

        let r = u / n;
        let c = u % n;

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
            for l in 1..=max_l {
                let nr = r as isize + DIR[d].0 * l as isize;
                let nc = c as isize + DIR[d].1 * l as isize;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                    break;
                }
                let v = nr as usize * n + nc as usize;
                if !floor[v] {
                    break;
                }

                if board.len(v) + 1 > 8 {
                    continue;
                }

                if ctx.visited[v] != token {
                    ctx.visited[v] = token;
                    ctx.dist[v] = ctx.dist[u] + 1;
                    ctx.prev[v] = u;
                    ctx.prev_action[v] = (k, d, l);
                    ctx.queue[tail] = v;
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
    // 逆順を反転
    ctx.actions[..action_count].reverse();
    Some(action_count)
}

fn move_single_trampoline_fast(
    n: usize,
    floor: &[bool],
    board: &mut StackBoard,
    plates: &[usize],
    ops: &mut Vec<Op>,
    start: usize,
    target: usize,
    ctx: &mut BfsContext,
) -> bool {
    if start == target {
        return true;
    }
    let Some(action_count) = find_path_single_trampoline_fast(n, floor, board, start, target, ctx) else {
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

        let r = p / n;
        let c = p % n;
        let nr = (r as isize + DIR[d].0 * l as isize) as usize;
        let nc = (c as isize + DIR[d].1 * l as isize) as usize;
        let next_p = nr * n + nc;

        if board.len(next_p) + (h - k) > 8 {
            return false;
        }

        ops.push(Op {
            r,
            c,
            k,
            d: DIR_CHAR[d],
            l,
        });

        // 移動と反転
        let mut temp = [0usize; 8];
        let block_len = h - k;
        for i in 0..block_len {
            temp[i] = board.stacks[p][k + i];
        }
        board.lens[p] = k;

        // 反転して next_p に追加
        let next_orig_len = board.lens[next_p];
        for i in 0..block_len {
            board.stacks[next_p][next_orig_len + i] = temp[block_len - 1 - i];
        }
        board.lens[next_p] += block_len;

        // 帰巣判定
        for &chk in &[p, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
                while board.last(chk) == Some(plate_color) {
                    board.pop(chk);
                }
            }
        }

        if board.is_empty(next_p) {
            break;
        }
    }
    true
}

// タワー輸送用のゼロアロケーションBFS
fn find_path_merge_fast(
    n: usize,
    floor: &[bool],
    board: &StackBoard,
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

        let r = u / n;
        let c = u % n;
        for d in 0..4 {
            let nr = r as isize + DIR[d].0;
            let nc = c as isize + DIR[d].1;
            if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                continue;
            }
            let v = nr as usize * n + nc as usize;
            if !floor[v] {
                continue;
            }

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
    floor: &[bool],
    board: &mut StackBoard,
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
    let Some(step_count) = find_path_merge_fast(n, floor, board, start, target, tower_h, ctx) else {
        return false;
    };

    let mut curr = start;
    for idx in 0..step_count {
        let d = ctx.path_tower[idx];
        let r = curr / n;
        let c = curr % n;
        let nr = (r as isize + DIR[d].0) as usize;
        let nc = (c as isize + DIR[d].1) as usize;
        let next_p = nr * n + nc;

        let k = 0;
        let l = 1;

        ops.push(Op {
            r,
            c,
            k,
            d: DIR_CHAR[d],
            l,
        });

        // タワー全体を反転して移動
        let h = board.len(curr);
        let mut temp = [0usize; 8];
        for i in 0..h {
            temp[i] = board.stacks[curr][i];
        }
        board.lens[curr] = 0;

        let next_orig_len = board.lens[next_p];
        for i in 0..h {
            board.stacks[next_p][next_orig_len + i] = temp[h - 1 - i];
        }
        board.lens[next_p] += h;

        for &p in &[curr, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                while board.last(p) == Some(plate_color) {
                    board.pop(p);
                }
            }
        }

        curr = next_p;
        if curr == target {
            break;
        }
    }
    true
}

// 巣へのダイレクト射出
fn try_direct_nest_launch_fast(
    n: usize,
    floor: &[bool],
    board: &mut StackBoard,
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

    for l in 1..=dist_l {
        let nr = r_p as isize + DIR[d].0 * l as isize;
        let nc = c_p as isize + DIR[d].1 * l as isize;
        let next_p = nr as usize * n + nc as usize;
        if !floor[next_p] {
            return false;
        }
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

    let block_len = h - k;
    let mut temp = [0usize; 8];
    for i in 0..block_len {
        temp[i] = board.stacks[p][k + i];
    }
    board.lens[p] = k;

    let target_orig_len = board.lens[target];
    for i in 0..block_len {
        board.stacks[target][target_orig_len + i] = temp[block_len - 1 - i];
    }
    board.lens[target] += block_len;

    for &chk in &[p, target] {
        if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
            while board.last(chk) == Some(plate_color) {
                board.pop(chk);
            }
        }
    }

    true
}

fn eject_nest_blockers_fast(
    n: usize,
    floor: &[bool],
    board: &mut StackBoard,
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

        let h = board.len(nest);
        let top_color = board.last(nest).unwrap();
        let mut count = 0;
        for i in (0..h).rev() {
            if board.stacks[nest][i] == top_color {
                count += 1;
            } else {
                break;
            }
        }

        let r = nest / n;
        let c = nest % n;
        let mut best_action = None;
        let mut min_load = usize::MAX;

        let k = h - count;
        let max_l = k + 1;

        for d in 0..4 {
            for l in 1..=max_l {
                let nr = r as isize + DIR[d].0 * l as isize;
                let nc = c as isize + DIR[d].1 * l as isize;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                    break;
                }
                let next_p = nr as usize * n + nc as usize;
                if !floor[next_p] {
                    break;
                }
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
            r,
            c,
            k,
            d: DIR_CHAR[d],
            l,
        });

        let mut temp = [0usize; 8];
        for i in 0..count {
            temp[i] = board.stacks[nest][k + i];
        }
        board.lens[nest] = k;

        let next_orig_len = board.lens[next_p];
        for i in 0..count {
            board.stacks[next_p][next_orig_len + i] = temp[count - 1 - i];
        }
        board.lens[next_p] += count;

        for &p in &[nest, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                while board.last(p) == Some(plate_color) {
                    board.pop(p);
                }
            }
        }

        eject_count += 1;
    }
}

// ゼロアロケーション・シミュレーション関数
fn solve_fast(
    n: usize,
    _k: usize,
    floor: &[bool],
    init_board: &StackBoard,
    plates: &[usize],
    dist_table: &[Vec<usize>],
    color_order: &[usize],
    group_max_size: usize,
    ctx: &mut BfsContext,
) -> (usize, Vec<Op>) {
    let mut board = *init_board; // 3.6KBの超高速memcpy
    let mut ops: Vec<Op> = Vec::with_capacity(300);

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

                // 巣へのダイレクト射出
                let mut launched = false;
                for i in 0..pos_count {
                    let p = positions[i];
                    if try_direct_nest_launch_fast(n, floor, &mut board, plates, &mut ops, p, color) {
                        launched = true;
                        progress = true;
                        break;
                    }
                }
                if launched {
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
                        let h = board.len(bp);
                        let top_c = board.last(bp).unwrap();
                        let mut cnt = 0;
                        for i in (0..h).rev() {
                            if board.stacks[bp][i] == top_c {
                                cnt += 1;
                            } else {
                                break;
                            }
                        }

                        let r = bp / n;
                        let c = bp % n;
                        let mut ejected = false;
                        for d in 0..4 {
                            let nr = r as isize + DIR[d].0;
                            let nc = c as isize + DIR[d].1;
                            if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                                continue;
                            }
                            let np = nr as usize * n + nc as usize;
                            if floor[np] && board.len(np) + cnt <= 8 {
                                let k_rem = h - cnt;
                                ops.push(Op {
                                    r,
                                    c,
                                    k: k_rem,
                                    d: DIR_CHAR[d],
                                    l: 1,
                                });

                                let mut temp = [0usize; 8];
                                for i in 0..cnt {
                                    temp[i] = board.stacks[bp][k_rem + i];
                                }
                                board.lens[bp] = k_rem;

                                let np_orig_len = board.lens[np];
                                for i in 0..cnt {
                                    board.stacks[np][np_orig_len + i] = temp[cnt - 1 - i];
                                }
                                board.lens[np] += cnt;

                                for &chk in &[bp, np] {
                                    if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                                        while board.last(chk) == Some(pc) {
                                            board.pop(chk);
                                        }
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

                // 巣から最も遠いスライムを選択
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

                // seed_p に近い順にソート
                let mut sorted_pos = positions[..pos_count].to_vec();
                sorted_pos.sort_by_key(|&p| dist_table[p][seed_p]);
                let group_size = pos_count.min(group_max_size);
                let group = &sorted_pos[..group_size];

                let mut best_x = target_plate;
                let mut min_cost = usize::MAX;

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

                    if possible && cost < min_cost {
                        min_cost = cost;
                        best_x = x;
                    }
                }

                for &p in group {
                    if p == best_x {
                        continue;
                    }
                    let success = move_single_trampoline_fast(n, floor, &mut board, plates, &mut ops, p, best_x, ctx);
                    if !success {
                        move_single_trampoline_fast(n, floor, &mut board, plates, &mut ops, p, target_plate, ctx);
                    }
                }

                if best_x != target_plate && !board.is_empty(best_x) {
                    let success = move_tower_merge_fast(n, floor, &mut board, plates, &mut ops, best_x, target_plate, ctx);
                    if !success {
                        let mut fallback_count = 0;
                        while !board.is_empty(best_x) && fallback_count < 10 {
                            let s = move_single_trampoline_fast(n, floor, &mut board, plates, &mut ops, best_x, target_plate, ctx);
                            if !s { break; }
                            fallback_count += 1;
                        }
                    }
                }

                eject_nest_blockers_fast(n, floor, &mut board, plates, &mut ops, target_plate, color);
            }
        }

        if !progress {
            for p in 0..n * n {
                if let Some(color) = board.last(p) {
                    let target_plate = plates[color];
                    move_single_trampoline_fast(n, floor, &mut board, plates, &mut ops, p, target_plate, ctx);
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

fn main() {
    let start_time = Instant::now();
    let time_limit = std::time::Duration::from_millis(1850);

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

    let dist_table = compute_all_pairs_dist(n, &floor);
    let mut rng = XorShift::new(20260925);
    let init_board = StackBoard::new(n, &init_stacks);
    let mut ctx = Box::new(BfsContext::new());

    let mut current_order: Vec<usize> = (0..k).collect();
    let mut current_group_size = 5;
    let (mut current_score, mut best_ops) = solve_fast(
        n,
        k,
        &floor,
        &init_board,
        &plates,
        &dist_table,
        &current_order,
        current_group_size,
        &mut ctx,
    );
    let mut best_score = current_score;

    let t_start: f64 = 8.0;
    let t_end: f64 = 0.5;

    let mut iter_count = 0;

    while start_time.elapsed() < time_limit {
        iter_count += 1;
        let elapsed_ratio = start_time.elapsed().as_secs_f64() / time_limit.as_secs_f64();
        let temperature = t_start * (1.0 - elapsed_ratio) + t_end * elapsed_ratio;

        let mut next_order = current_order.clone();
        let mut next_group_size = current_group_size;

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
        } else if move_type < 85 {
            let mut i = rng.gen_range(0, k);
            let mut j = rng.gen_range(0, k);
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            next_order[i..=j].reverse();
        } else {
            next_group_size = rng.gen_range(3, 9);
        }

        let (cand_score, cand_ops) = solve_fast(
            n,
            k,
            &floor,
            &init_board,
            &plates,
            &dist_table,
            &next_order,
            next_group_size,
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
            current_group_size = next_group_size;
            current_score = cand_score;

            if cand_score < best_score {
                best_score = cand_score;
                best_ops = cand_ops;
            }
        }
    }

    eprintln!("Iterations: {}, Best Score: {}", iter_count, best_score);

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &best_ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
