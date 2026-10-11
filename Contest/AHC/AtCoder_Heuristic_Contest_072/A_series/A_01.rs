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

    let mut ops: Vec<Op> = Vec::new();

    // 全スライムが帰巣するまで繰り返す
    loop {
        // スタックの一番上にスライムがいるマスを探す
        // できるだけ巣に近いスライムを選ぶ（マンハッタン距離等で優先）
        let mut best_target: Option<(usize, usize)> = None;
        let mut min_dist = usize::MAX;

        for p in 0..n * n {
            if let Some(&color) = stacks[p].last() {
                let target_p = plates[color];
                let dist = (p / n).abs_diff(target_p / n) + (p % n).abs_diff(target_p % n);
                if dist < min_dist {
                    min_dist = dist;
                    best_target = Some((p, color));
                }
            }
        }

        let Some((start_p, color)) = best_target else {
            break; // 全てのスライムが帰巣完了
        };

        let target_plate = plates[color];

        // start_p から target_plate への最短経路を BFS で探索
        let mut dist = vec![usize::MAX; n * n];
        let mut prev = vec![usize::MAX; n * n];
        let mut prev_dir = vec![0; n * n];
        let mut q = VecDeque::new();

        dist[start_p] = 0;
        q.push_back(start_p);

        while let Some(curr) = q.pop_front() {
            if curr == target_plate {
                break;
            }

            let r = curr / n;
            let c = curr % n;

            for d in 0..4 {
                let nr = r as isize + DIR[d].0;
                let nc = c as isize + DIR[d].1;
                if nr < 0 || nr >= n as isize || nc < 0 || nc >= n as isize {
                    continue;
                }
                let next_p = nr as usize * n + nc as usize;
                if !floor[next_p] {
                    continue;
                }
                // 高さ8を超えるマスには入れない（最上部の1匹を乗せるため、現在の高さが8未満であること）
                if stacks[next_p].len() >= 8 {
                    continue;
                }

                if dist[next_p] > dist[curr] + 1 {
                    dist[next_p] = dist[curr] + 1;
                    prev[next_p] = curr;
                    prev_dir[next_p] = d;
                    q.push_back(next_p);
                }
            }
        }

        if dist[target_plate] == usize::MAX {
            eprintln!("Warning: No path to target plate for color {}", color);
            break;
        }

        // 経路復元
        let mut path_dirs = Vec::new();
        let mut curr = target_plate;
        while curr != start_p {
            let p = prev[curr];
            let d = prev_dir[curr];
            path_dirs.push(d);
            curr = p;
        }
        path_dirs.reverse();

        // 経路上を1マスずつ進める
        let mut curr_p = start_p;
        for d in path_dirs {
            let h = stacks[curr_p].len();
            assert!(h > 0);
            let k = h - 1; // 一番上の1匹だけを跳ばし、下のk匹は残す
            let l = 1;     // 1マスジャンプ

            let r = curr_p / n;
            let c = curr_p % n;
            let next_r = (r as isize + DIR[d].0) as usize;
            let next_c = (c as isize + DIR[d].1) as usize;
            let next_p = next_r * n + next_c;

            ops.push(Op {
                r,
                c,
                k,
                d: DIR_CHAR[d],
                l,
            });

            // 盤面状態の更新
            let slime = stacks[curr_p].pop().unwrap();
            stacks[next_p].push(slime);

            // 出発点と着地点での帰巣判定
            for &p in &[curr_p, next_p] {
                if let Some(plate_color) = plates.iter().position(|&plate_pos| plate_pos == p) {
                    while stacks[p].last() == Some(&plate_color) {
                        stacks[p].pop();
                    }
                }
            }

            curr_p = next_p;

            // もし運んでいるスライムが着地点（巣）で帰巣して消えたら、このスライムの移動は終了
            if curr_p == target_plate {
                break;
            }
        }
    }

    let out = io::stdout();
    let mut writer = io::BufWriter::new(out.lock());
    for op in &ops {
        writeln!(writer, "{} {} {} {} {}", op.r, op.c, op.k, op.d, op.l).unwrap();
    }
}
