from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
source = (root / 'B_series/B_09.rs').read_text(encoding='utf-8')
source = '// C_01: B_09 + differential validation + complete bridges + compact exact search.\n' + source
start = source.index('pub fn merge_straight_jumps(')
end = source.index('// -------------------------------------------------------------\n// 閉路・往復', start)
part = source[start:end]
old_trace = '''        let mut prefix_boards = Vec::with_capacity(n_ops + 1);
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
        }'''
assert part.count(old_trace) == 5
part = part.replace(old_trace, '''        let deadline = deadline_at(start_time, deadline_ms);
        let Some(trace) = TraceCache::build(initial_board, &ops, grid, deadline) else { break; };
        let prefix_boards = &trace.boards;
        let mut replacement = Vec::with_capacity(64);''')
blocks = list(re.finditer(r'(?P<indent> +)let mut test_ops = ops.clone\(\);(?P<body>.*?)\n(?P=indent)if let Some\(cleared\) = simulate_all\(\*initial_board, &test_ops, grid\) \{\n +if cleared >= target_cleared \{\n +ops = test_ops;\n +improved = true;\n +break \'outer;\n +\}\n(?P=indent)\}', part, re.S))
assert len(blocks) == 5, len(blocks)
for index, m in reversed(list(enumerate(blocks))):
    indent = m['indent']
    if index == 0:
        body = '''replacement.clear();
let mut changed = op_i;
changed.l = total_l; changed.k = k_cand;
replacement.push(changed);
replacement.extend_from_slice(&ops[i+1..j]);'''
    elif index == 1:
        body = '''replacement.clear();
replacement.push(Op { r:op_i.r,c:op_i.c,k:k_cand,d:d_dir,l:dist_l });
replacement.extend_from_slice(&ops[i+1..j]);'''
    else:
        body = m['body']
        cutoff = body.find('for _ in') if index < 4 else body.find('let remove_cnt')
        assert cutoff >= 0
        body = body[:cutoff]
        assignments = re.findall(r'test_ops\[i(?: \+ [12])?\] = (Op \{.*?\});', body, re.S)
        assert len(assignments) == (3 if index == 4 else 2), assignments
        body = 'replacement.clear();\n' + '\n'.join('replacement.push(' + op + ');' for op in assignments)
    body += '''
match validate_patch(&ops, &trace, i, j+1, &replacement, grid, target_cleared, deadline) {
    CheckResult::Accept => {
        record_patch(j+1-i, replacement.len());
        ops.splice(i..j+1, replacement.iter().copied());
        improved = true;
        break 'outer;
    }
    CheckResult::BudgetExpired => break 'outer,
    CheckResult::Reject => {}
}'''
    replacement = '\n'.join(indent + line for line in body.splitlines())
    part = part[:m.start()] + replacement + part[m.end():]
source = source[:start] + part + source[end:]

# Preserve the legacy fast-only pipeline; A-C get explicit stage budgets.
source = source.replace('        const TOTAL_DEADLINE_MS: u128 = 1750;', '''        let mode = crate::b01::post_mode();
        let total_begin = Instant::now();
        let raw_len = ops_b01.len();
        let TOTAL_DEADLINE_MS: u128 = if mode == "fast" { 1750 } else { 1650 };''')
needle = '        let ops_lns = if start_time.elapsed().as_millis() + 150 < TOTAL_DEADLINE_MS {'
assert source.count(needle) == 1
source = source.replace(needle, '''        eprintln!("PHASE: name=geometry elapsed_ms={:.3} before={} after={}", total_begin.elapsed().as_secs_f64()*1000.0, raw_len, cur_ops.len());
        if mode != "fast" {
            let final_ops = crate::b01::finish_new_pipeline(cur_ops, &b01_board, &b01_grid, start_time, &mode);
            eprintln!("CANDIDATE: raw={} final={}", raw_len, final_ops.len());
            return final_ops;
        }
''' + needle)

# Keep a validated raw fallback even if SA finishes after the processing deadline.
old = '    let mut best_final_ops: Option<Vec<crate::b01::Op>> = None;'
assert source.count(old) == 1
source = source.replace(old, '''    let fallback: Vec<crate::b01::Op> = sa_best_ops.iter().map(|op| crate::b01::Op {
        r:op.r,c:op.c,k:op.k,d:match op.d {'U'=>0,'D'=>1,'L'=>2,_=>3},l:op.l,
    }).collect();
    let mut best_final_ops = if crate::b01::valid_complete(&b01_board,&fallback,&b01_grid) {
        Some(fallback)
    } else { None };
    let pool_candidates = candidate_pools.len();
    let mut processed_candidates = 0;''')
source = source.replace('        let cleaned = run_pipeline(cand_ops);', '        let cleaned = run_pipeline(cand_ops);\n        processed_candidates += 1;')
source = source.replace('        if best_final_ops.as_ref().map_or(true, |b| cleaned.len() < b.len()) {',
    '        if crate::b01::valid_complete(&b01_board,&cleaned,&b01_grid) && best_final_ops.as_ref().map_or(true, |b| cleaned.len() < b.len()) {')
source = source.replace('    let final_cleaned_ops = best_final_ops.unwrap();', '''    let final_cleaned_ops = best_final_ops.expect("no validated candidate");
    eprintln!("POOL: raw_best_ops={} pool_candidates={} processed_candidates={}",sa_best_ops.len(),pool_candidates,processed_candidates);''')
source = source.replace('    if use_a44 {\n', '''    eprintln!("BRANCH: {}", if use_a44 {"A44"} else {"B01"});
    if use_a44 {
''')
source = source.replace('fn main() {\n', 'fn main() {\n    let execution_start = std::time::Instant::now();\n')
source = source.replace('        b01::solve(&input);\n    }\n}', '        b01::solve(&input);\n    }\n    b01::print_patch_stats();\n    eprintln!("TIME: elapsed_ms={:.3}",execution_start.elapsed().as_secs_f64()*1000.0);\n}')

# Move pruning onto the same cache and patch checker.
pstart = source.index('pub fn prune_cycle_and_uturn_ops(')
pend = source.index('// -------------------------------------------------------------\n// 過去改変 第3弾:', pstart)
source = source[:pstart] + (root/'scratch/c01_prune.rs').read_text(encoding='utf-8') + '\n' + source[pend:]
insert = source.index('// -------------------------------------------------------------\n// 過去改変 第2弾:')
fragment = (root/'scratch/c01_postprocess.rs').read_text(encoding='utf-8')
fragment += '\npub fn valid_complete(initial: &BitBoard,ops: &[Op],grid: &Grid) -> bool { simulate_all(*initial,ops,grid)==Some(grid.initial_slimes) }\n'
fragment += (root/'scratch/c01_tests.rs').read_text(encoding='utf-8')
source = source[:insert] + fragment + '\n' + source[insert:]
source = source.replace('let TOTAL_DEADLINE_MS: u128', 'let total_deadline_ms: u128').replace('TOTAL_DEADLINE_MS', 'total_deadline_ms')
(root/'C_01.rs').write_text(source,encoding='utf-8')
print('Built C_01.rs')
