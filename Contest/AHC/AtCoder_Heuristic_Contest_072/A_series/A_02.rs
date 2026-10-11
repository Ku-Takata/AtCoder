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

// 現在の盤面状況下で、追加の高さ add_h を持ったスライム/タワーが start から target へ移動する最短経路を探索
fn find_path(
    n: usize,
    floor: &[bool],
    stacks: &[Vec<usize>],
    start: usize,
    target: usize,
    add_h: usize,
    only_empty_intermediates: bool,
) -> Option<Vec<usize>> {
    let mut dist = vec![usize::MAX; n * n];
    let mut prev = vec![usize::MAX; n * n];
    let mut prev_dir = vec![0; n * n];
    let mut q = VecDeque::new();

    dist[start] = 0;
    q.push_back(start);

    while let Some(u) = q.pop_front() {
        if u == target {
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
            // 途中のマスで他のスライムがいるマスを踏まないようにする
            if only_empty_intermediates && v != target && !stacks[v].is_empty() {
                continue;
            }
            // 着地後の高さが8以下であること
            if stacks[v].len() + add_h > 8 {
                continue;
            }
            if dist[v] > dist[u] + 1 {
                dist[v] = dist[u] + 1;
                prev[v] = u;
                prev_dir[v] = d;
                q.push_back(v);
            }
        }
    }

    if dist[target] == usize::MAX {
        return None;
    }

    let mut path = Vec::new();
    let mut curr = target;
    while curr != start {
        path.push(prev_dir[curr]);
        curr = prev[curr];
    }
    path.reverse();
    Some(path)
}

// 1匹のスライムを start から target へ運ぶ
fn move_single(
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
    // まずは空きマスのみを通る安全な経路を試し、なければ他スライムの上を通過する経路を許可
    let path = find_path(n, floor, stacks, start, target, 1, true)
        .or_else(|| find_path(n, floor, stacks, start, target, 1, false));

    let Some(path) = path else {
        return false;
    };
    let mut curr = start;
    for d in path {
        let h = stacks[curr].len();
        if h == 0 {
            break;
        }
        let k = h - 1;
        let l = 1;
        let r = curr / n;
        let c = curr % n;
        let nr = (r as isize + DIR[d].0) as usize;
        let nc = (c as isize + DIR[d].1) as usize;
        let next_p = nr * n + nc;

        ops.push(Op {
            r,
            c,
            k,
            d: DIR_CHAR[d],
            l,
        });

        let slime = stacks[curr].pop().unwrap();
        stacks[next_p].push(slime);

        // 帰巣判定（出発点と着地点）
        for &p in &[curr, next_p] {
            if let Some(plate_color) = plates.iter().position(|&pos| pos == p) {
                while stacks[p].last() == Some(&plate_color) {
                    stacks[p].pop();
                }
            }
        }

        curr = next_p;
        // もし途中で巣に入って消滅したら移動終了
        if stacks[curr].is_empty() {
            break;
        }
    }
    true
}

// タワー全体（stacks[start] のすべて）を target へ運ぶ
fn move_tower(
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
    // タワー輸送時は他のスライムの巻き込みを防ぐため必ず空きマスのみを通過する
    let Some(path) = find_path(n, floor, stacks, start, target, tower_h, true) else {
        return false;
    };
    let mut curr = start;
    for d in path {
        let _h = stacks[curr].len();
        let k = 0; // タワー全体
        let l = 1;
        let r = curr / n;
        let c = curr % n;
        let nr = (r as isize + DIR[d].0) as usize;
        let nc = (c as isize + DIR[d].1) as usize;
        let next_p = nr * n + nc;

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

    // 各色ごとに順番にスライムをグループ化・集約して巣へ運ぶ
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
            if positions.is_empty() {
                break;
            }

            // 巣から最も遠いスライムをシードとして選択
            positions.sort_by_key(|&p| std::cmp::Reverse(dist_table[p][target_plate]));
            let seed_p = positions[0];

            // seed_p に近い順に最大8匹を選んでグループ化
            positions.sort_by_key(|&p| dist_table[p][seed_p]);
            let group_size = positions.len().min(8);
            let group = positions[..group_size].to_vec();

            // 最適な集約点 X を探索
            let mut best_x = target_plate;
            let mut min_cost = usize::MAX;

            for x in 0..n * n {
                if !floor[x] {
                    continue;
                }
                // 他の色のスライムがいるマスは集約点に選ばない
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

            // グループ内のスライムを best_x に集める
            for &p in &group {
                if p == best_x {
                    continue;
                }
                let success = move_single(n, &floor, &mut stacks, &plates, &mut ops, p, best_x);
                if !success {
                    // best_x への集約が失敗した場合は直接巣へ運ぶフォールバック
                    move_single(n, &floor, &mut stacks, &plates, &mut ops, p, target_plate);
                }
            }

            // best_x に集まったタワーを巣へ一括輸送
            if best_x != target_plate && !stacks[best_x].is_empty() {
                let success = move_tower(n, &floor, &mut stacks, &plates, &mut ops, best_x, target_plate);
                if !success {
                    // タワー輸送が通らない場合は 1匹ずつ個別配送
                    while !stacks[best_x].is_empty() {
                        move_single(n, &floor, &mut stacks, &plates, &mut ops, best_x, target_plate);
                    }
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
