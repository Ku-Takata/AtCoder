use std::collections::VecDeque;
use std::io::{self, Read, Write};

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

// トランポリン跳躍（l <= k+1）を活用した1匹移動の最短経路探索
fn find_path_single_trampoline(
    n: usize,
    floor: &[bool],
    stacks: &[Vec<usize>],
    start: usize,
    target: usize,
) -> Option<Vec<(usize, usize, usize, usize)>> {
    let mut dist = vec![usize::MAX; n * n];
    let mut prev = vec![usize::MAX; n * n];
    let mut prev_action = vec![(0, 0, 0); n * n]; // (k, d, l)
    let mut q = VecDeque::new();

    dist[start] = 0;
    q.push_back(start);

    while let Some(u) = q.pop_front() {
        if u == target {
            break;
        }

        let r = u / n;
        let c = u % n;

        // u でのタワーの高さ
        let h = if u == start {
            stacks[start].len()
        } else {
            stacks[u].len() + 1 // 運んできた1匹が乗る
        };

        if h == 0 {
            continue;
        }

        let k = h - 1; // 最上部の1匹だけを跳ばす
        let max_l = k + 1; // 最大飛距離

        for d in 0..4 {
            for l in 1..=max_l {
                let nr = r as isize + DIR[d].0 * l as isize;
                let nc = c as isize + DIR[d].1 * l as isize;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                    break; // 盤面外に出たらこれ以上遠くには行けない
                }
                let v = nr as usize * n + nc as usize;
                if !floor[v] {
                    break; // 壁にぶつかったらこれ以上遠くには行けない
                }

                // 着地点 v の高さチェック（1匹乗せるので stacks[v].len() + 1 <= 8）
                if stacks[v].len() + 1 > 8 {
                    continue; // ここには着地できないが、飛び越えてさらに先へ行くことは可能
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
    while curr != start {
        let p = prev[curr];
        let (k, d, l) = prev_action[curr];
        actions.push((p, k, d, l));
        curr = p;
    }
    actions.reverse();
    Some(actions)
}

// トランポリン跳躍を使って1匹のスライムを運ぶ
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

    for (p, k, d, l) in actions {
        let h = stacks[p].len();
        if h == 0 || k >= h {
            break;
        }

        let r = p / n;
        let c = p % n;
        let nr = (r as isize + DIR[d].0 * l as isize) as usize;
        let nc = (c as isize + DIR[d].1 * l as isize) as usize;
        let next_p = nr * n + nc;

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

        // 帰巣判定（出発点と着地点）
        for &chk in &[p, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == chk) {
                while stacks[chk].last() == Some(&plate_color) {
                    stacks[chk].pop();
                }
            }
        }

        if stacks[next_p].is_empty() {
            break; // 巣で帰巣して消滅
        }
    }
    true
}

// 巻き込みを考慮したタワー輸送用最短経路探索（途中で他スライムを巻き込んで高さが増加することを追跡）
fn find_path_merge(
    n: usize,
    floor: &[bool],
    stacks: &[Vec<usize>],
    start: usize,
    target: usize,
    start_h: usize,
) -> Option<Vec<usize>> {
    let mut dist = vec![vec![usize::MAX; 9]; n * n];
    let mut prev = vec![vec![(usize::MAX, 0, 0); 9]; n * n]; // (prev_u, prev_h, dir)
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
    while curr_u != start || curr_h != start_h {
        let (pu, ph, d) = prev[curr_u][curr_h];
        path.push(d);
        curr_u = pu;
        curr_h = ph;
    }
    path.reverse();
    Some(path)
}

// 巻き込みを許可してタワー全体を target へ運ぶ
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

        let k = 0; // タワー全体を跳ばす
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

        // 帰巣判定
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

// 巣に巻き込まれて到着した異色スライムを退避させ、下にある目的色スライムを一気に帰巣させる
fn eject_nest_blockers(
    n: usize,
    floor: &[bool],
    stacks: &mut [Vec<usize>],
    plates: &[usize],
    ops: &mut Vec<Op>,
    nest: usize,
    nest_color: usize,
) {
    while !stacks[nest].is_empty() {
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

        // 隣接する床マスで退避可能なマスを探す（トランポリン跳躍も考慮）
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
            eprintln!("Warning: No space to eject blockers from nest {}", nest);
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

        // 出発点（nest）と着地点（next_p）での帰巣処理
        for &p in &[nest, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                while stacks[p].last() == Some(&plate_color) {
                    stacks[p].pop();
                }
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut lines = input.lines();

    let first_line = lines.next().unwrap();
    let mut parts = first_line.split_whitespace();
    let n: usize = parts.next().unwrap().parse().unwrap();
    let k: usize = parts.next().unwrap().parse().unwrap();

    let mut floor = vec![true; n * n];
    let mut stacks: Vec<Vec<usize>> = vec![Vec::new(); n * n];
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
                    stacks[p].push(color);
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
    let mut ops: Vec<Op> = Vec::new();

    // 盤面上の全スライムが帰巣するまで周回処理
    loop {
        let mut total_slimes = 0;
        for p in 0..n * n {
            total_slimes += stacks[p].len();
        }
        if total_slimes == 0 {
            break;
        }

        let mut progress = false;

        // 各色ごとに順番にスライムを処理
        for color in 0..k {
            let target_plate = plates[color];

            loop {
                // 盤面上の色 color のスライムが存在するマスを収集
                let mut positions = Vec::new();
                for p in 0..n * n {
                    if stacks[p].last() == Some(&color) {
                        positions.push(p);
                    }
                }

                // 最上部に見当たらないが、スタックの下に埋もれている場合
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
                                let k = h - cnt;
                                ops.push(Op {
                                    r,
                                    c,
                                    k,
                                    d: DIR_CHAR[d],
                                    l: 1,
                                });
                                let mut block = stacks[bp].split_off(k);
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

                // 巻き込みの余裕を持たせるため、初期グループサイズは最大5匹
                positions.sort_by_key(|&p| dist_table[p][seed_p]);
                let group_size = positions.len().min(5);
                let group = positions[..group_size].to_vec();

                // 最適な集約点 X を探索
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
                    let success = move_single_trampoline(n, &floor, &mut stacks, &plates, &mut ops, p, best_x);
                    if !success {
                        move_single_trampoline(n, &floor, &mut stacks, &plates, &mut ops, p, target_plate);
                    }
                }

                // best_x に集まったタワーを巣へ一括輸送
                if best_x != target_plate && !stacks[best_x].is_empty() {
                    let success = move_tower_merge(n, &floor, &mut stacks, &plates, &mut ops, best_x, target_plate);
                    if !success {
                        while !stacks[best_x].is_empty() {
                            move_single_trampoline(n, &floor, &mut stacks, &plates, &mut ops, best_x, target_plate);
                        }
                    }
                }

                // 巣にフタをしている異色スライムがあれば退避させ、下にある目的色スライムを一気に帰巣させる
                eject_nest_blockers(n, &floor, &mut stacks, &plates, &mut ops, target_plate, color);
            }
        }

        if !progress {
            // 最終安全網: 残っている最上部スライムを1匹ずつ運ぶ
            for p in 0..n * n {
                if let Some(&color) = stacks[p].last() {
                    let target_plate = plates[color];
                    move_single_trampoline(n, &floor, &mut stacks, &plates, &mut ops, p, target_plate);
                    break;
                }
            }
        }
    }

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
