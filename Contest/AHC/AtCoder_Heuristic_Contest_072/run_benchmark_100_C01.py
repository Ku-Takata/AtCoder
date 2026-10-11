"""One sequential paired evaluation; never selects per-seed winners or retunes code."""
import argparse
import csv
import hashlib
import json
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DEFAULT_INPUTS = Path(r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in')
DEFAULT_VIS = Path(r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe')


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def compile_source(source, exe, log):
    command = ['rustc', '--edition=2021', '-O', str(source), '-o', str(exe)]
    result = subprocess.run(command, capture_output=True, text=True, cwd=ROOT)
    log.write_text('COMMAND: ' + repr(command) + '\n' + result.stdout + result.stderr, encoding='utf-8')
    if result.returncode:
        raise RuntimeError(f'Compilation failed; see {log}')
    return sha256(source)


def run_case(name, seed, inputs, vis, run_dir, hashes):
    folder = run_dir / name
    input_file = inputs / f'{seed:04d}.txt'
    output_file = folder / f'{seed:04d}.txt'
    stderr_file = folder / f'{seed:04d}.stderr.txt'
    timed_out = False
    start = time.perf_counter()
    with input_file.open('rb') as fin, output_file.open('wb') as fout, stderr_file.open('wb') as ferr:
        try:
            process = subprocess.run([str(run_dir / 'binaries' / f'{name}.exe')],
                                     stdin=fin, stdout=fout, stderr=ferr, timeout=3.0, cwd=ROOT)
            returncode = process.returncode
        except subprocess.TimeoutExpired:
            timed_out, returncode = True, None
    elapsed_ms = (time.perf_counter() - start) * 1000
    # subprocess.wait(timeout) on Windows may poll coarsely. The native solver's
    # final timestamp is retained separately; this is the external wall time.
    lines = output_file.read_text(encoding='utf-8').splitlines()
    total_ops = len([line for line in lines if line.strip()])
    score, remaining, valid = None, None, False
    validation_code = None
    if not timed_out and returncode == 0:
        checked = subprocess.run([str(vis), str(input_file), str(output_file), '--no-vis'],
                                 capture_output=True, text=True, timeout=30, cwd=run_dir)
        validation_code = checked.returncode
        (folder / f'{seed:04d}.validator.stdout.txt').write_text(checked.stdout, encoding='utf-8')
        (folder / f'{seed:04d}.validator.stderr.txt').write_text(checked.stderr, encoding='utf-8')
        # vis prints Score=0 even after an illegal move, with an error preceding it.
        match = re.fullmatch(r'\s*Score\s*=\s*(\d+)\s*', checked.stdout)
        if checked.returncode == 0 and match:
            score = int(match[1])
            delta = score - total_ops
            if delta >= 0 and delta % 100000 == 0:
                remaining, valid = delta // 100000, True
    raw = input_file.read_text(encoding='utf-8').split()
    slimes = sum(c.islower() for line in raw[2:] for c in line)
    walls = sum(c == '#' for line in raw[2:] for c in line)
    branch = 'A44' if slimes <= 120 or (slimes <= 145 and walls < 100) else 'B01'
    stderr = stderr_file.read_text(encoding='utf-8', errors='replace')
    sa = re.search(r'sa_iters=(\d+)', stderr)
    native_time = re.search(r'^TIME: elapsed_ms=([\d.]+)',stderr,re.M)
    patch = re.search(r'^PATCH: (.*)$', stderr, re.M)
    metrics = {k: int(v) for k, v in re.findall(r'(\w+)=(\d+)', patch[1])} if patch else {}
    phases = []
    for line in stderr.splitlines():
        if line.startswith(('PHASE:', 'PHASE_STATS:', 'POOL:', 'CANDIDATE:')):
            phases.append(line)
    return dict(seed=f'{seed:04d}', model=name, branch=branch, source_hash=hashes[name],
                input_hash=sha256(input_file), valid=valid, remaining=remaining,
                total_ops=total_ops, score=score, elapsed_ms=elapsed_ms,
                over_2s=elapsed_ms >= 2000, timed_out=timed_out, returncode=returncode,
                validator_returncode=validation_code, sa_iters=int(sa[1]) if sa else None,
                native_elapsed_ms=float(native_time[1]) if native_time else None,
                metrics=metrics, phases=phases)


def summarize(rows):
    result = {}
    for name in ['B_09', 'C_01']:
        subset = [r for r in rows if r['model'] == name]
        all_valid = all(r['valid'] and r['remaining'] == 0 for r in subset)
        result[name] = dict(cases=len(subset), invalid=sum(not r['valid'] for r in subset),
            incomplete=sum(r['valid'] and r['remaining'] != 0 for r in subset),
            timeouts=sum(r['timed_out'] for r in subset), over_2s=sum(r['over_2s'] for r in subset),
            total_ops=sum(r['total_ops'] for r in subset) if all_valid else None,
            max_elapsed_ms=max(r['elapsed_ms'] for r in subset),
            mean_elapsed_ms=sum(r['elapsed_ms'] for r in subset)/len(subset),
            metrics={key:sum(r['metrics'].get(key, 0) for r in subset)
                     for key in ['trials','accepted','saved_ops','replay_steps','rejoin_accepts','compact_nodes','budget_stops']})
    pairs = [(rows[i], rows[i+1]) for i in range(0, len(rows), 2)]
    pairs = [{r['model']: r for r in pair} for pair in pairs]
    valid_pairs = [p for p in pairs if all(r['valid'] and r['remaining']==0 for r in p.values())]
    diffs = [p['C_01']['total_ops'] - p['B_09']['total_ops'] for p in valid_pairs]
    result['comparison'] = dict(valid_pairs=len(diffs), wins=sum(d<0 for d in diffs),
                                ties=sum(d==0 for d in diffs), losses=sum(d>0 for d in diffs))
    for group in ['A44','B01','excluding_seed0']:
        subset = [p for p in valid_pairs if (p['C_01']['seed']!='0000' if group=='excluding_seed0' else p['C_01']['branch']==group)]
        result['comparison'][group] = dict(cases=len(subset),
            B_09=sum(p['B_09']['total_ops'] for p in subset), C_01=sum(p['C_01']['total_ops'] for p in subset))
    return result


def write_report(run_dir, rows, summary, hashes, compiler):
    columns = ['seed','model','branch','source_hash','input_hash','valid','remaining','total_ops','score','elapsed_ms','over_2s','timed_out','returncode','validator_returncode','sa_iters']
    with (run_dir/'results.csv').open('w', encoding='utf-8', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=columns, extrasaction='ignore')
        writer.writeheader(); writer.writerows(rows)
    (run_dir/'results.json').write_text(json.dumps(dict(compiler=compiler, hashes=hashes, summary=summary, cases=rows),indent=2),encoding='utf-8')
    b, c = summary['B_09'], summary['C_01']
    comp = summary['comparison']
    text = ['# C_01 / B_09 100ケース比較', '',
            '2026-10-03。0000～0099を各1回、1プロセスずつ実行。偶数seedはB_09→C_01、奇数seedはC_01→B_09。',
            '両者とも `rustc --edition=2021 -O`。全出力を公式vis.exeの`--no-vis`で検証。小さい手数が良い。', '',
            '| 指標 | B_09 | C_01 |', '|---|---:|---:|',
            f"| 合計手数 | {b['total_ops']} | {c['total_ops']} |",
            f"| 不正出力 | {b['invalid']} | {c['invalid']} |",
            f"| 未帰巣あり | {b['incomplete']} | {c['incomplete']} |",
            f"| 3秒で打ち切り | {b['timeouts']} | {c['timeouts']} |",
            f"| 外部計測2秒以上 | {b['over_2s']} | {c['over_2s']} |",
            f"| 平均実行時間ms | {b['mean_elapsed_ms']:.2f} | {c['mean_elapsed_ms']:.2f} |",
            f"| 最大実行時間ms | {b['max_elapsed_ms']:.2f} | {c['max_elapsed_ms']:.2f} |", '']
    if b['total_ops'] is not None and c['total_ops'] is not None:
        diff = c['total_ops']-b['total_ops']
        text += [f"同条件での差: {diff:+d}手 ({diff/b['total_ops']*100:+.3f}%)。", '']
    text += [f"C_01の勝ち / 同点 / 負け: {comp['wins']} / {comp['ties']} / {comp['losses']}（有効ペア{comp['valid_pairs']}）。", '',
             '| 内訳 | ケース数 | B_09 | C_01 | 差 |', '|---|---:|---:|---:|---:|']
    for group in ['A44','B01','excluding_seed0']:
        g=comp[group];text.append(f"| {group} | {g['cases']} | {g['B_09']} | {g['C_01']} | {g['C_01']-g['B_09']:+d} |")
    text += ['', '指南書記載の過去B_09は32,194手（今回の再計測とは別）。']
    if c['total_ops'] is not None:
        text += [f"C_01の過去基準との差: {c['total_ops']-32194:+d}手。"]
    text += ['', '外部時間は起動・終了を含む壁時計時間。この環境の測定であり、提出先でのTLEを保証するものではない。',
             '評価結果に基づくコード改良・パラメータ調整・再評価は行っていない。', '',
             '## 実装と確認', '',
             '- 差分区間検証を既存の5種類の幾何学的短縮とprune/cycleに適用。合流時に再生を打ち切る。',
             '- 空き中継点も含む3～8手→2手の置換。帰巣後の高さで全ての合法kを列挙。',
             '- 12マス以下の局所状態を反復深化DFSで探索。3～6手窓、最大4手・2,000ノード・1ms/窓。',
             '- 最大24手区間の独立操作を保存し、端点集合の閉包を固定点まで計算。',
             '- SAは1,400msを維持。geometryは1,650msまで、bridgeは最大30ms、compactは最大50ms、終了1,750ms。',
             '- 候補プールの内容重複判定・公平配分（改善D）は今回の変更に含めない。',
             '- 6つの単体テストが成功。局所遷移と全盤面物理の比較、差分検証と全再生の比較を含む。',
             '- `--post=fast` / `--post=bridge` / `--post=local-contiguous` / `--post=local`（標準）で切替。今回の100ケースは標準のみ。', '',
             '## 集計ログ', '', '```json', json.dumps(c['metrics'], indent=2), '```', '',
             '## 再現情報', '', f'コンパイラ: `{compiler.strip()}`', '',
             f"B_09 SHA-256: `{hashes['B_09']}`", '', f"C_01 SHA-256: `{hashes['C_01']}`", '',
             'ケースごとの標準出力・stderr・公式検証器ログは各モデルのサブフォルダ。results.csv/results.jsonに詳細。']
    (run_dir/'REPORT.md').write_text('\n'.join(text)+'\n',encoding='utf-8')


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--inputs',type=Path,default=DEFAULT_INPUTS)
    parser.add_argument('--vis',type=Path,default=DEFAULT_VIS)
    parser.add_argument('--run-dir',type=Path,default=ROOT/'outputs/C_01_comparison_20261003')
    parser.add_argument('--resume',action='store_true',help='Continue completed pairs without changing or rebuilding the solvers')
    args=parser.parse_args()
    args.inputs=args.inputs.resolve();args.vis=args.vis.resolve();run_dir=args.run_dir.resolve()
    if run_dir.exists() and not args.resume:
        raise RuntimeError('Run directory already exists. Choose a new --run-dir to preserve previous measurements.')
    for seed in range(100):
        if not (args.inputs/f'{seed:04d}.txt').is_file(): raise FileNotFoundError(seed)
    if not args.vis.is_file(): raise FileNotFoundError(args.vis)
    run_dir.mkdir(parents=True,exist_ok=args.resume)
    for folder in ['binaries','B_09','C_01','inputs']: (run_dir/folder).mkdir(exist_ok=args.resume)
    compiler=subprocess.run(['rustc','--version'],capture_output=True,text=True,check=True).stdout
    hashes={}
    rows=json.loads((run_dir/'progress.json').read_text(encoding='utf-8')) if args.resume else []
    if len(rows)%2: raise RuntimeError('Incomplete progress metadata')
    for name,source in [('B_09',ROOT/'B_series/B_09.rs'),('C_01',ROOT/'C_01.rs')]:
        if args.resume:
            hashes[name]=sha256(source)
            if any(r['source_hash']!=hashes[name] for r in rows if r['model']==name):
                raise RuntimeError('Source changed; cannot resume')
        else:
            hashes[name]=compile_source(source,run_dir/'binaries'/f'{name}.exe',run_dir/f'{name}.compile.txt')
    # Snapshot all inputs and the official validator so an external folder move
    # cannot interrupt the rest of the paired run.
    for seed in range(100):
        path=args.inputs/f'{seed:04d}.txt'
        if any(r['input_hash']!=sha256(path) for r in rows if r['seed']==f'{seed:04d}'):
            raise RuntimeError('Input changed; cannot resume')
        shutil.copy2(path,run_dir/'inputs'/path.name)
    shutil.copy2(args.vis,run_dir/'binaries/vis.exe')
    start_seed=len(rows)//2
    if args.resume:
        for name in ['B_09','C_01']:
            for path in (run_dir/name).glob(f'{start_seed:04d}.*'):
                shutil.copy2(path,path.with_name(path.name+'.interrupted'))
        (run_dir/'interruption.txt').write_text(f'Original Downloads tools folder was moved during case {start_seed:04d}. Completed pairs were kept. The unfinished pair was restarted using unchanged solver binaries; interrupted output was preserved.\n',encoding='utf-8')
    start=time.perf_counter()
    for seed in range(start_seed,100):
        order=['B_09','C_01'] if seed%2==0 else ['C_01','B_09']
        for name in order: rows.append(run_case(name,seed,run_dir/'inputs',run_dir/'binaries/vis.exe',run_dir,hashes))
        (run_dir/'progress.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
        if (seed+1)%10==0:
            print(f'{seed+1}/100 pairs completed; elapsed={time.perf_counter()-start:.1f}s',flush=True)
    summary=summarize(rows)
    write_report(run_dir,rows,summary,hashes,compiler)
    print(json.dumps(summary,indent=2),flush=True)
    print(f'Report: {run_dir / "REPORT.md"}',flush=True)
    bad=any(not r['valid'] or r['remaining']!=0 or r['over_2s'] for r in rows)
    return 1 if bad else 0


if __name__=='__main__':
    sys.exit(main())
