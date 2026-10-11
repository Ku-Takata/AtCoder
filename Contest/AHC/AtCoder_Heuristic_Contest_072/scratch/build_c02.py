from pathlib import Path
import hashlib

root=Path(__file__).resolve().parents[1]
baseline=root/'C_01.rs'
expected='B5BDC5F64924C97E4AD37F4FB1788AAC5EAD35B6D1D464226832CF7E7384FD5A'
assert hashlib.sha256(baseline.read_bytes()).hexdigest().upper()==expected, 'C_01 source differs from specification'
source=baseline.read_text(encoding='utf-8')
source='// C_02: C_01 checkpoint + bounded tail-only compact search and window cache.\n'+source
old='pub fn solve(input_str: &str, params: &crate::TunableParams) {'
assert source.count(old)==1
source=source.replace(old,'pub fn solve(input_str: &str, params: &crate::TunableParams, main_tail_deadline: Instant) {')
old='    let final_cleaned_ops = best_final_ops.expect("no validated candidate");'
assert source.count(old)==1
source=source.replace(old,'''    let checkpoint_ops = best_final_ops.expect("no validated candidate");
    let checkpoint_score = checkpoint_ops.len();
    eprintln!("CHECKPOINT: ops={} elapsed_ms={:.3} sa_iters={} processed_candidates={}",checkpoint_score,start_time.elapsed().as_secs_f64()*1000.0,sa_iter_count,processed_candidates);
    let tail_deadline = main_tail_deadline.min(start_time + std::time::Duration::from_millis(1745));
    let final_cleaned_ops = crate::b01::tail_optimize(checkpoint_ops,&b01_board,&b01_grid,tail_deadline);
    assert!(final_cleaned_ops.len()<=checkpoint_score);''')
source=source.replace('        a44::solve(&input, &global_params);','        a44::solve(&input, &global_params, execution_start + std::time::Duration::from_millis(1745));')
fragment=(root/'scratch/c02_tail.rs').read_text(encoding='utf-8')
fragment+='\n'+(root/'scratch/c02_tests.rs').read_text(encoding='utf-8')
insert=source.index('// -------------------------------------------------------------\n// 過去改変 第2弾:')
source=source[:insert]+fragment+'\n'+source[insert:]
(root/'C_02.rs').write_text(source,encoding='utf-8')
print('Built C_02.rs; preserved C_01 hash')
