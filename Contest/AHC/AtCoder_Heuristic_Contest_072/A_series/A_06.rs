use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::time::Instant;

const DIR: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const DIR_CHAR: [char; 4] = ['U', 'D', 'L', 'R'];

#[derive(Clone, Debug)]
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
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn gen_range(&mut self, l: usize, r: usize) -> usize {
        l + (self.next_u64() as usize % (r - l))
    }
    fn gen_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

// 盤面上の全床マス間の最短経路距離テーブルを計算
fn compute_all_pairs_dist(n: usize, floor: &[bool]) -> Vec<Vec<usize>> {
    let mut dist = vec![vec![usize::MAX; n * n]; n * n];
    for start in 0..n * n {
        if !floor[start] {
            continue;
        }
        dist[start][start] = 0;
        let mut q = VecDeque::new();
        q.push_back(start);
        while let Some(u) = q.pop_front() {
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
                    q.push_back(v);
                }
            }
        }
    }
    dist
}

// トランポリン跳躍を活用した1匹移動の最短経路探索（他色スライムを踏み台にして大ジャンプ）
fn find_path_single_trampoline(
    n: usize,
    floor: &[bool],
    stacks: &[Vec<usize>],
    start: usize,
    target: usize,
) -> Option<Vec<(usize, usize, usize, usize)>> {
    if start == target {
        return Some(Vec::new());
    }

    let mut dist = vec![usize::MAX; n * n];
    let mut prev = vec![usize::MAX; n * n];
    let mut prev_action = vec![(0, 0, 0); n * n];
    let mut q = VecDeque::new();

    dist[start] = 0;
    q.push_back(start);

    while let Some(u) = q.pop_front() {
        if u == target {
            break;
        }

        let r = u / n;
        let c = u % n;

        let h = if u == start {
            stacks[start].len()
        } else {
            stacks[u].len() + 1
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

                if stacks[v].len() + 1 > 8 {
                    continue;
                }

                if dist[v] > dist[u] + 1 {
                    dist[v] = dist[u] + 1;
                    prev[v] = u;
                    prev_action[v] = (k, d, l);
                    q.push_back(v);
                }
            }
        }
    }

    if dist[target] == usize::MAX {
        return None;
    }

    let mut actions = Vec::new();
    let mut curr = target;
    let mut steps = 0;
    while curr != start && steps < n * n {
        let p = prev[curr];
        if p == usize::MAX {
            return None;
        }
        let (k, d, l) = prev_action[curr];
        actions.push((p, k, d, l));
        curr = p;
        steps += 1;
    }
    actions.reverse();
    Some(actions)
}

fn move_single_trampoline(
    n: usize,
    floor: &[bool],
    stacks: &mut [Vec<usize>],
    plates: &[usize],
    ops: &mut Vec<Op>,
    start: usize,
    target: usize,
) -> bool {
    if start == target {
        return true;
    }
    let Some(actions) = find_path_single_trampoline(n, floor, stacks, start, target) else {
        return false;
    };
    if actions.is_empty() {
        return true;
    }

    for (p, k, d, l) in actions {
        let h = stacks[p].len();
        if h == 0 || k >= h {
            return false;
        }

        let r = p / n;
        let c = p % n;
        let nr = (r as isize + DIR[d].0 * l as isize) as usize;
        let nc = (c as isize + DIR[d].1 * l as isize) as usize;
        let next_p = nr * n + nc;

        if stacks[next_p].len() + (h - k) > 8 {
            return false;
        }

        ops.push(Op {
            r,
            c,
            k,
            d: DIR_CHAR[d],
            l,
        });

        let mut block = stacks[p].split_off(k);
        block.reverse();
        stacks[next_p].extend(block);

        for &chk in &[p, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
                while stacks[chk].last() == Some(&plate_color) {
                    stacks[chk].pop();
                }
            }
        }

        if stacks[next_p].is_empty() {
            break;
        }
    }
    true
}

// 巻き込みを考慮したタワー輸送用最短経路探索
fn find_path_merge(
    n: usize,
    floor: &[bool],
    stacks: &[Vec<usize>],
    start: usize,
    target: usize,
    start_h: usize,
) -> Option<Vec<usize>> {
    let mut dist = vec![vec![usize::MAX; 9]; n * n];
    let mut prev = vec![vec![(usize::MAX, 0, 0); 9]; n * n];
    let mut q = VecDeque::new();

    dist[start][start_h] = 0;
    q.push_back((start, start_h));

    let mut best_target_h = None;

    while let Some((u, h)) = q.pop_front() {
        if u == target {
            best_target_h = Some(h);
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
                if stacks[v].len() + h > 8 {
                    continue;
                }
                h
            } else {
                let nh = h + stacks[v].len();
                if nh > 8 {
                    continue;
                }
                nh
            };

            if dist[v][next_h] > dist[u][h] + 1 {
                dist[v][next_h] = dist[u][h] + 1;
                prev[v][next_h] = (u, h, d);
                q.push_back((v, next_h));
            }
        }
    }

    let Some(mut curr_h) = best_target_h else {
        return None;
    };

    let mut path = Vec::new();
    let mut curr_u = target;
    let mut steps = 0;
    while (curr_u != start || curr_h != start_h) && steps < n * n * 9 {
        let (pu, ph, d) = prev[curr_u][curr_h];
        if pu == usize::MAX {
            return None;
        }
        path.push(d);
        curr_u = pu;
        curr_h = ph;
        steps += 1;
    }
    path.reverse();
    Some(path)
}

fn move_tower_merge(
    n: usize,
    floor: &[bool],
    stacks: &mut [Vec<usize>],
    plates: &[usize],
    ops: &mut Vec<Op>,
    start: usize,
    target: usize,
) -> bool {
    let tower_h = stacks[start].len();
    if tower_h == 0 || start == target {
        return true;
    }
    let Some(path) = find_path_merge(n, floor, stacks, start, target, tower_h) else {
        return false;
    };
    let mut curr = start;
    for d in path {
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

        let mut block = stacks[curr].split_off(0);
        block.reverse();
        stacks[next_p].extend(block);

        for &p in &[curr, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                while stacks[p].last() == Some(&plate_color) {
                    stacks[p].pop();
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

// 巣から直接直線上で狙えるスライムを、1手でダイレクトに巣へ射出する
fn try_direct_nest_launch(
    n: usize,
    floor: &[bool],
    stacks: &mut [Vec<usize>],
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
            (3, c_t - c_p) // Right
        } else {
            (2, c_p - c_t) // Left
        }
    } else if c_p == c_t {
        if r_t > r_p {
            (1, r_t - r_p) // Down
        } else {
            (0, r_p - r_t) // Up
        }
    } else {
        return false; // 直線上にない
    };

    // 途中のマスがすべて床か確認
    for l in 1..=dist_l {
        let nr = r_p as isize + DIR[d].0 * l as isize;
        let nc = c_p as isize + DIR[d].1 * l as isize;
        let next_p = nr as usize * n + nc as usize;
        if !floor[next_p] {
            return false;
        }
    }

    let h = stacks[p].len();
    if dist_l > h {
        return false;
    }

    let k = dist_l - 1;
    if stacks[target].len() + (h - k) > 8 {
        return false;
    }

    ops.push(Op {
        r: r_p,
        c: c_p,
        k,
        d: DIR_CHAR[d],
        l: dist_l,
    });

    let mut block = stacks[p].split_off(k);
    block.reverse();
    stacks[target].extend(block);

    for &chk in &[p, target] {
        if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
            while stacks[chk].last() == Some(&plate_color) {
                stacks[chk].pop();
            }
        }
    }

    true
}

fn eject_nest_blockers(
    n: usize,
    floor: &[bool],
    stacks: &mut [Vec<usize>],
    plates: &[usize],
    ops: &mut Vec<Op>,
    nest: usize,
    nest_color: usize,
) {
    let mut eject_count = 0;
    while !stacks[nest].is_empty() && eject_count < 10 {
        if stacks[nest].last() == Some(&nest_color) {
            stacks[nest].pop();
            continue;
        }

        let h = stacks[nest].len();
        let top_color = *stacks[nest].last().unwrap();
        let mut count = 0;
        for &c in stacks[nest].iter().rev() {
            if c == top_color {
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
                if stacks[next_p].len() + count <= 8 {
                    if stacks[next_p].len() < min_load {
                        min_load = stacks[next_p].len();
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

        let mut block = stacks[nest].split_off(k);
        block.reverse();
        stacks[next_p].extend(block);

        for &p in &[nest, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                while stacks[p].last() == Some(&plate_color) {
                    stacks[p].pop();
                }
            }
        }

        eject_count += 1;
    }
}

// 指定されたパラメータでシミュレーションを実行するソルバー関数
fn solve(
    n: usize,
    _k: usize,
    floor: &[bool],
    init_stacks: &[Vec<usize>],
    plates: &[usize],
    dist_table: &[Vec<usize>],
    color_order: &[usize],
    group_max_size: usize,
) -> (usize, Vec<Op>) {
    let mut stacks = init_stacks.to_vec();
    let mut ops: Vec<Op> = Vec::new();

    let mut outer_loops = 0;
    loop {
        outer_loops += 1;
        if outer_loops > 20 {
            break;
        }

        let mut total_slimes = 0;
        for p in 0..n * n {
            total_slimes += stacks[p].len();
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

                let mut positions = Vec::new();
                for p in 0..n * n {
                    if stacks[p].last() == Some(&color) {
                        positions.push(p);
                    }
                }

                // 1. 直線上で直接巣に大ジャンプできるスライムがいれば優先射出！
                let mut launched = false;
                for &p in &positions {
                    if try_direct_nest_launch(n, floor, &mut stacks, plates, &mut ops, p, color) {
                        launched = true;
                        progress = true;
                        break;
                    }
                }
                if launched {
                    continue;
                }

                if positions.is_empty() {
                    let mut buried_p = None;
                    for p in 0..n * n {
                        if stacks[p].contains(&color) {
                            buried_p = Some(p);
                            break;
                        }
                    }
                    if let Some(bp) = buried_p {
                        let h = stacks[bp].len();
                        let top_c = *stacks[bp].last().unwrap();
                        let mut cnt = 0;
                        for &c in stacks[bp].iter().rev() {
                            if c == top_c {
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
                            if floor[np] && stacks[np].len() + cnt <= 8 {
                                let k_rem = h - cnt;
                                ops.push(Op {
                                    r,
                                    c,
                                    k: k_rem,
                                    d: DIR_CHAR[d],
                                    l: 1,
                                });
                                let mut block = stacks[bp].split_off(k_rem);
                                block.reverse();
                                stacks[np].extend(block);
                                for &chk in &[bp, np] {
                                    if let Some(pc) = plates.iter().position(|&pos| pos == chk) {
                                        while stacks[chk].last() == Some(&pc) {
                                            stacks[chk].pop();
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

                // 巣から最も遠いスライムをシードとして選択
                positions.sort_by_key(|&p| std::cmp::Reverse(dist_table[p][target_plate]));
                let seed_p = positions[0];

                positions.sort_by_key(|&p| dist_table[p][seed_p]);
                let group_size = positions.len().min(group_max_size);
                let group = positions[..group_size].to_vec();

                let mut best_x = target_plate;
                let mut min_cost = usize::MAX;

                for x in 0..n * n {
                    if !floor[x] {
                        continue;
                    }
                    if !stacks[x].is_empty() && !group.contains(&x) && x != target_plate {
                        continue;
                    }
                    if dist_table[x][target_plate] == usize::MAX {
                        continue;
                    }

                    let mut cost = dist_table[x][target_plate];
                    let mut possible = true;
                    for &p in &group {
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

                // グループ内のスライムをトランポリン跳躍を活用して best_x に集める
                for &p in &group {
                    if p == best_x {
                        continue;
                    }
                    let success = move_single_trampoline(n, floor, &mut stacks, plates, &mut ops, p, best_x);
                    if !success {
                        move_single_trampoline(n, floor, &mut stacks, plates, &mut ops, p, target_plate);
                    }
                }

                // best_x に集まったタワーを巣へ一括輸送
                if best_x != target_plate && !stacks[best_x].is_empty() {
                    let success = move_tower_merge(n, floor, &mut stacks, plates, &mut ops, best_x, target_plate);
                    if !success {
                        let mut fallback_count = 0;
                        while !stacks[best_x].is_empty() && fallback_count < 10 {
                            let s = move_single_trampoline(n, floor, &mut stacks, plates, &mut ops, best_x, target_plate);
                            if !s { break; }
                            fallback_count += 1;
                        }
                    }
                }

                eject_nest_blockers(n, floor, &mut stacks, plates, &mut ops, target_plate, color);
            }
        }

        if !progress {
            for p in 0..n * n {
                if let Some(&color) = stacks[p].last() {
                    let target_plate = plates[color];
                    move_single_trampoline(n, floor, &mut stacks, plates, &mut ops, p, target_plate);
                    break;
                }
            }
        }
    }

    let unreturned: usize = stacks.iter().map(|s| s.len()).sum();
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

    let mut current_order: Vec<usize> = (0..k).collect();
    let mut current_group_size = 5;
    let (mut current_score, mut best_ops) = solve(
        n,
        k,
        &floor,
        &init_stacks,
        &plates,
        &dist_table,
        &current_order,
        current_group_size,
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

        let (cand_score, cand_ops) = solve(
            n,
            k,
            &floor,
            &init_stacks,
            &plates,
            &dist_table,
            &next_order,
            next_group_size,
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
