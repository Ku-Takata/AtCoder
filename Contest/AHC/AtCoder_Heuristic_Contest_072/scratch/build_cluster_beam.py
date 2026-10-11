import os

with open('A_series/A_44.rs', 'r', encoding='utf-8') as f:
    a44_code = f.read()

with open('B_series/B_01.rs', 'r', encoding='utf-8') as f:
    b01_code = f.read()

# Modify A_44 to expose necessary functions
# Make BitBoard, BfsContext, pre_cluster_full_towers, etc. public
a44_mod = a44_code.replace(
    "struct BitBoard", "pub struct BitBoard"
).replace(
    "struct Op", "pub struct Op"
).replace(
    "struct BfsContext", "pub struct BfsContext"
).replace(
    "fn new(n: usize", "pub fn new(n: usize"
).replace(
    "fn new() -> Self", "pub fn new() -> Self"
).replace(
    "fn compute_max_ray", "pub fn compute_max_ray"
).replace(
    "fn compute_all_pairs_dist", "pub fn compute_all_pairs_dist"
).replace(
    "fn pre_cluster_full_towers", "pub fn pre_cluster_full_towers"
).replace(
    "    r: usize,\n    c: usize,\n    k: usize,\n    d: char,\n    l: usize,",
    "    pub r: usize,\n    pub c: usize,\n    pub k: usize,\n    pub d: char,\n    pub l: usize,"
).replace(
    "    r: usize,\r\n    c: usize,\r\n    k: usize,\r\n    d: char,\r\n    l: usize,",
    "    pub r: usize,\r\n    pub c: usize,\r\n    pub k: usize,\r\n    pub d: char,\r\n    pub l: usize,"
).replace(
    "fn main() {", "fn disabled_main() {"
)

# In B_01:
# Modify main to run pre-clustering before chokudai_search
b01_main_replacement = """
fn main() {
    let params = parse_args();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut lines = input.lines().filter(|l| !l.trim().is_empty());

    let first_line = lines.next().unwrap();
    let mut tokens = first_line.split_whitespace();
    let n: usize = tokens.next().unwrap().parse().unwrap();
    let k: usize = tokens.next().unwrap().parse().unwrap();

    let mut raw_grid = Vec::with_capacity(n);
    let mut floor = vec![true; n * n];
    let mut init_stacks: Vec<Vec<usize>> = vec![Vec::new(); n * n];
    let mut plates = vec![0; k];

    for i in 0..n {
        let line = lines.next().unwrap().trim();
        raw_grid.push(line.chars().collect::<Vec<char>>());
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

    let (grid, initial_board) = Grid::new(n, k, &raw_grid);

    // =========================================================
    // 方針 B: A_44 の本格的事前クラスタリングを実行
    // =========================================================
    let dir_step: [isize; 4] = [-(n as isize), n as isize, -1, 1];
    let max_ray = a44::compute_max_ray(n, &floor);
    let dist_table = a44::compute_all_pairs_dist(n, &floor);
    let mut a44_board = a44::BitBoard::new(n, &init_stacks);
    let mut ctx = Box::new(a44::BfsContext::new());
    let mut prefix_ops_a44 = Vec::new();

    // 全色について事前クラスタリング（近くの同色スライムを合体させてタワー化）
    for color in 0..k {
        a44::pre_cluster_full_towers(
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

    // A_44 の Op を B_01 の Op に変換
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

    // クラスタリング後の盤面を作成
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

    // クラスタリング後の盤面から Chokudai Search をスタート！
    let search_ops = chokudai_search(start_board, &grid, &params);

    // prefix_ops と search_ops を連結
    let mut all_ops = applied_prefix_ops;
    all_ops.extend(search_ops);

    // 全手順に対して不要手削除（Pruning）
    let final_ops = prune_ops(all_ops, &initial_board, &grid, grid.initial_slimes);

    for op in &final_ops {
        println!("{} {} {} {} {}", op.r, op.c, op.k, DIR_CHAR[op.d], op.l);
    }
}
"""

b01_mod = b01_code[:b01_code.rfind("fn main() {")] + b01_main_replacement

cluster_beam_source = f"""// -------------------------------------------------------------
// 方針 B: Chokudai Search への本格的事前クラスタリング注入
// A_44 の pre_cluster_full_towers で綺麗なタワーを作ってから
// B_01 の高速 Chokudai Search をスタートさせるハイブリッド探索
// -------------------------------------------------------------

mod a44 {{
{a44_mod}
}}

{b01_mod}
"""

with open('cluster_beam_test.rs', 'w', encoding='utf-8') as f:
    f.write(cluster_beam_source)

print("Successfully generated cluster_beam_test.rs!")
