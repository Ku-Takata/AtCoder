with open('B_series/B_03.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# 1. 冒頭ヘッダーの整理 (lines 0..60)
header = """// -------------------------------------------------------------
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
"""

# 2. pre_cluster_dense の抽出
pre_cluster_dense_lines = []
capturing = False
depth = 0
started_brace = False
for l in lines:
    if 'pub fn pre_cluster_dense(' in l:
        capturing = True
    if capturing:
        pre_cluster_dense_lines.append(l)
        if '{' in l:
            started_brace = True
        if started_brace:
            depth += l.count('{') - l.count('}')
            if depth == 0:
                break

pre_cluster_dense_code = ''.join(pre_cluster_dense_lines)
# move_smart_transfer 呼び出しに dist_table を追加
pre_cluster_dense_code = pre_cluster_dense_code.replace(
    'move_smart_transfer(n, dir_step, max_ray, board, plates, ops, u, best_parent, ctx);',
    'move_smart_transfer(n, dir_step, max_ray, board, plates, ops, u, best_parent, dist_table, ctx);'
)

# 3. a44 モジュールの抽出と修正 (lines 60..3336)
a44_code = ''.join(lines[60:3336])

beam_start = a44_code.find('// -------------------------------------------------------------\n// フェーズ 1 用: ビームサーチ関数群')
solve_start = a44_code.find('pub fn solve(input_str: &str, params: &crate::TunableParams) {')
assert beam_start != -1 and solve_start != -1

a44_cleaned = a44_code[:beam_start] + pre_cluster_dense_code + '\n' + a44_code[solve_start:]

# a44 内の Op 定義を pub 化
old_op = """struct Op {
    r: usize,
    c: usize,
    k: usize,
    d: char,
    l: usize,
}"""
new_op = """pub struct Op {
    pub r: usize,
    pub c: usize,
    pub k: usize,
    pub d: char,
    pub l: usize,
}"""
assert old_op in a44_cleaned
a44_cleaned = a44_cleaned.replace(old_op, new_op, 1)

a44_cleaned = a44_cleaned.replace('struct BitBoard {', 'pub struct BitBoard {')
a44_cleaned = a44_cleaned.replace('    fn new(n: usize, init_stacks: &[Vec<usize>]) -> Self {', '    pub fn new(n: usize, init_stacks: &[Vec<usize>]) -> Self {')
a44_cleaned = a44_cleaned.replace('struct BfsContext {', 'pub struct BfsContext {')
a44_cleaned = a44_cleaned.replace('    fn new() -> Self {', '    pub fn new() -> Self {')
a44_cleaned = a44_cleaned.replace('fn compute_max_ray(', 'pub fn compute_max_ray(')
a44_cleaned = a44_cleaned.replace('fn compute_all_pairs_dist(', 'pub fn compute_all_pairs_dist(')

a44_cleaned = a44_cleaned.replace('    let total_time_limit = std::time::Duration::from_millis(1850);\n    let phase1_limit = std::time::Duration::from_millis(600);\n', '')
a44_cleaned = a44_cleaned.replace('    let default_strategies = [0u8; 16];\n', '')

# 4. b01 モジュールの抽出と修正 (lines 7546..9312)
b01_code = ''.join(lines[7546:9312])
b01_cleaned = b01_code.replace('crate::cb::a44::', 'crate::a44::')

# 5. main 関数 (lines 9312..)
main_code = ''.join(lines[9312:])

# 組み立て
final_code = header + '\n' + a44_cleaned + '\n' + b01_cleaned + '\n' + main_code

with open('B_series/B_03_clean.rs', 'w', encoding='utf-8') as f:
    f.write(final_code)

print(f'Original lines: {len(lines)}, Cleaned lines: {len(final_code.splitlines())}')
