use std::fs;

const DIR: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Op {
    pub r: usize,
    pub c: usize,
    pub k: usize,
    pub d: usize,
    pub l: usize,
}

#[derive(Clone, Copy)]
pub struct BitBoard {
    pub lens: [u8; 400],
    pub colors: [u32; 400],
    pub occupied_mask: [u64; 7],
}

impl BitBoard {
    pub fn new() -> Self {
        Self {
            lens: [0; 400],
            colors: [0; 400],
            occupied_mask: [0; 7],
        }
    }

    #[inline(always)]
    pub fn set_occupied(&mut self, p: usize) {
        self.occupied_mask[p / 64] |= 1u64 << (p % 64);
    }

    #[inline(always)]
    pub fn clear_occupied(&mut self, p: usize) {
        self.occupied_mask[p / 64] &= !(1u64 << (p % 64));
    }

    #[inline(always)]
    pub fn transfer_and_reverse(&mut self, from: usize, to: usize, k: usize) {
        let h_from = self.lens[from] as usize;
        let jumping = h_from - k;
        let c_from = self.colors[from];

        let mut flying_reversed = 0u32;
        let mut tmp = c_from >> (4 * k);
        for _ in 0..jumping {
            flying_reversed = (flying_reversed << 4) | (tmp & 0xF);
            tmp >>= 4;
        }

        let mask_keep = if k == 0 { 0 } else { (1u32 << (4 * k)) - 1 };
        self.colors[from] = c_from & mask_keep;
        self.lens[from] = k as u8;

        let h_to = self.lens[to] as usize;
        self.colors[to] |= flying_reversed << (4 * h_to);
        self.lens[to] = (h_to + jumping) as u8;
    }

    #[inline(always)]
    pub fn clean_nest(&mut self, p: usize, target_color: usize) -> usize {
        let mut h = self.lens[p] as usize;
        if h == 0 {
            return 0;
        }
        let mut cleared = 0;
        let mut c = self.colors[p];
        while h > 0 {
            let top_col = ((c >> (4 * (h - 1))) & 0xF) as usize;
            if top_col == target_color {
                cleared += 1;
                h -= 1;
                c &= (1u32 << (4 * h)) - 1;
            } else {
                break;
            }
        }
        self.lens[p] = h as u8;
        self.colors[p] = c;
        cleared
    }
}

pub struct Grid {
    pub n: usize,
    pub k: usize,
    pub cell_count: usize,
    pub is_floor: [bool; 400],
    pub nest_pos: [usize; 16],
    pub nest_color_at: [Option<usize>; 400],
    pub max_reach: [[usize; 4]; 400],
    pub initial_slimes: usize,
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

        let mut max_reach = [[0usize; 4]; 400];
        for p in 0..cell_count {
            if !is_floor[p] {
                continue;
            }
            let r = p / n;
            let c = p % n;
            for d in 0..4 {
                let (dr, dc) = DIR[d];
                let mut reach = 0;
                let mut cr = r as isize + dr;
                let mut cc = c as isize + dc;
                while cr >= 0 && cr < n as isize && cc >= 0 && cc < n as isize {
                    let np = cr as usize * n + cc as usize;
                    if !is_floor[np] {
                        break;
                    }
                    reach += 1;
                    cr += dr;
                    cc += dc;
                }
                max_reach[p][d] = reach;
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
                max_reach,
                initial_slimes,
            },
            board,
        )
    }
}

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

// -------------------------------------------------------------
// 直交2段ジャンプ（2-Hop Orthogonal Warp）
// -------------------------------------------------------------
pub fn orthogonal_2hop_jumps(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
) -> Vec<Op> {
    let mut improved = true;
    while improved {
        improved = false;
        let n_ops = ops.len();

        let mut prefix_boards = Vec::with_capacity(n_ops + 1);
        prefix_boards.push(*initial_board);
        let mut cur = *initial_board;
        let mut ok = true;
        for &op in &ops {
            if apply_op(&mut cur, op, grid).is_none() {
                ok = false;
                break;
            }
            prefix_boards.push(cur);
        }
        if !ok {
            break;
        }

        'outer: for i in 0..n_ops {
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
                    continue; // 同一直線は shortcut_ray_jumps が担当
                }

                // 2つのコーナー候補
                let corners = [
                    (op_i.r, dest_c), // 横 -> 縦
                    (dest_r, op_i.c), // 縦 -> 横
                ];

                for &(cr, cc) in &corners {
                    let p_corner = cr * grid.n + cc;
                    if !grid.is_floor[p_corner] {
                        continue;
                    }

                    // 第1手: (op_i.r, op_i.c) -> (cr, cc)
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

                    // 第2手: (cr, cc) -> (dest_r, dest_c)
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

                    // 元の手数: 手 i から 手 j までの手数 = (j - i + 1)
                    // 新手数: 2手
                    // (j - i + 1) > 2 であるため、成功すれば確実に短縮！
                    let mut test_ops = ops.clone();
                    test_ops[i] = Op {
                        r: op_i.r,
                        c: op_i.c,
                        k: k1,
                        d: d1,
                        l: l1,
                    };
                    test_ops[i + 1] = Op {
                        r: cr,
                        c: cc,
                        k: k2,
                        d: d2,
                        l: l2,
                    };
                    // 手 i+2 から 手 j までの (j - i - 1) 手を削除
                    for _ in 0..(j - i - 1) {
                        test_ops.remove(i + 2);
                    }

                    if let Some(cleared) = simulate_all(*initial_board, &test_ops, grid) {
                        if cleared >= target_cleared {
                            ops = test_ops;
                            improved = true;
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    ops
}

pub fn prune_ops(
    mut ops: Vec<Op>,
    initial_board: &BitBoard,
    grid: &Grid,
    target_cleared: usize,
) -> Vec<Op> {
    if ops.is_empty() {
        return ops;
    }
    for _pass in 0..2 {
        let mut improved = false;
        let mut prefix_boards = Vec::with_capacity(ops.len() + 1);
        let mut prefix_cleared = Vec::with_capacity(ops.len() + 1);
        prefix_boards.push(*initial_board);
        prefix_cleared.push(0);

        let mut cur_board = *initial_board;
        let mut cur_c = 0;
        let mut valid_ops = true;
        for &op in &ops {
            if let Some(c) = apply_op(&mut cur_board, op, grid) {
                cur_c += c;
                prefix_boards.push(cur_board);
                prefix_cleared.push(cur_c);
            } else {
                valid_ops = false;
                break;
            }
        }
        if !valid_ops {
            return ops;
        }

        let mut i = ops.len();
        while i > 0 {
            i -= 1;
            let mut test_board = prefix_boards[i];
            let mut test_cleared = prefix_cleared[i];
            let mut valid = true;

            for &op in &ops[(i + 1)..] {
                if let Some(c) = apply_op(&mut test_board, op, grid) {
                    test_cleared += c;
                } else {
                    valid = false;
                    break;
                }
            }

            if valid && test_cleared >= target_cleared {
                ops.remove(i);
                improved = true;
                prefix_boards.clear();
                prefix_cleared.clear();
                prefix_boards.push(*initial_board);
                prefix_cleared.push(0);
                let mut b = *initial_board;
                let mut c = 0;
                for &op in &ops {
                    if let Some(cleared) = apply_op(&mut b, op, grid) {
                        c += cleared;
                        prefix_boards.push(b);
                        prefix_cleared.push(c);
                    }
                }
            }
        }
        if !improved {
            break;
        }
    }
    ops
}

fn load_input(seed: usize) -> (Grid, BitBoard) {
    let path = format!("C:\\Users\\kuon9\\Downloads\\BrsBAXF2\\tools\\in\\{:04}.txt", seed);
    let s = fs::read_to_string(&path).expect("Failed to read input");
    let mut lines = s.lines();
    let first = lines.next().unwrap();
    let mut parts = first.split_whitespace();
    let n: usize = parts.next().unwrap().parse().unwrap();
    let k: usize = parts.next().unwrap().parse().unwrap();
    let mut raw_grid = Vec::new();
    for _ in 0..n {
        let line = lines.next().unwrap();
        raw_grid.push(line.chars().collect());
    }
    Grid::new(n, k, &raw_grid)
}

fn load_ops(seed: usize) -> Vec<Op> {
    let path = format!("outputs\\B_05\\{:04}.txt", seed);
    let s = fs::read_to_string(&path).expect("Failed to read output");
    let mut ops = Vec::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let r: usize = parts[0].parse().unwrap();
        let c: usize = parts[1].parse().unwrap();
        let k: usize = parts[2].parse().unwrap();
        let d_char: char = parts[3].chars().next().unwrap();
        let d = match d_char {
            'U' => 0,
            'D' => 1,
            'L' => 2,
            'R' => 3,
            _ => panic!("unknown dir"),
        };
        let l: usize = parts[4].parse().unwrap();
        ops.push(Op { r, c, k, d, l });
    }
    ops
}

fn main() {
    let mut total_orig = 0;
    let mut total_new = 0;
    let mut improved_seeds = 0;

    println!("Testing orthogonal_2hop_jumps on outputs/B_05...");

    for seed in 0..100 {
        let (grid, initial_board) = load_input(seed);
        let orig_ops = load_ops(seed);
        let orig_len = orig_ops.len();
        total_orig += orig_len;

        let mut cur_ops = orig_ops;
        for _ in 0..2 {
            let before = cur_ops.len();
            cur_ops = orthogonal_2hop_jumps(cur_ops, &initial_board, &grid, grid.initial_slimes);
            cur_ops = prune_ops(cur_ops, &initial_board, &grid, grid.initial_slimes);
            if cur_ops.len() == before {
                break;
            }
        }
        let new_len = cur_ops.len();
        total_new += new_len;

        if new_len < orig_len {
            improved_seeds += 1;
            println!("Seed {:04}: {} -> {} (diff: -{})", seed, orig_len, new_len, orig_len - new_len);
        }
    }

    println!("\n=== RESULTS ===");
    println!("Total Orig: {}", total_orig);
    println!("Total New:  {}", total_new);
    println!("Net Diff:   -{}", total_orig - total_new);
    println!("Improved seeds: {} / 100", improved_seeds);
}
