"""One fixed paired run of C_01 versus C_02; no automatic tuning or reruns."""
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

ROOT=Path(__file__).resolve().parent
OLD_RUN=ROOT/'outputs/C_01_comparison_20261003'
C01_HASH='B5BDC5F64924C97E4AD37F4FB1788AAC5EAD35B6D1D464226832CF7E7384FD5A'


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def fields(line):
    values={}
    for key,value in re.findall(r'(\w+)=([^\s]+)',line):
        try:values[key]=float(value) if '.' in value else int(value)
        except ValueError:values[key]=value
    return values


def compile_source(name,folder):
    source=ROOT/f'{name}.rs'
    command=['rustc','--edition=2021','-O',str(source),'-o',str(folder/'binaries'/f'{name}.exe')]
    result=subprocess.run(command,capture_output=True,text=True,cwd=ROOT)
    (folder/f'{name}.compile.txt').write_text(repr(command)+'\n'+result.stdout+result.stderr,encoding='utf-8')
    if result.returncode:raise RuntimeError(f'{name} compilation failed')
    return sha256(source)


def run_case(name,seed,folder,hashes,tail_mode):
    model_dir=folder/name
    infile=folder/'inputs'/f'{seed:04d}.txt'
    outfile=model_dir/f'{seed:04d}.txt'
    stderr_file=model_dir/f'{seed:04d}.stderr.txt'
    command=[str(folder/'binaries'/f'{name}.exe')]
    if name=='C_02':command.append(f'--tail={tail_mode}')
    timeout=False;returncode=None
    start=time.perf_counter()
    with infile.open('rb') as fin,outfile.open('wb') as fout,stderr_file.open('wb') as ferr:
        try:returncode=subprocess.run(command,stdin=fin,stdout=fout,stderr=ferr,timeout=3.0,cwd=ROOT).returncode
        except subprocess.TimeoutExpired:timeout=True
    elapsed_ms=(time.perf_counter()-start)*1000
    total_ops=sum(bool(line.strip()) for line in outfile.read_text(encoding='utf-8').splitlines())
    valid=False;remaining=None;score=None;validator_returncode=None
    if returncode==0 and not timeout:
        checked=subprocess.run([str(folder/'binaries/vis.exe'),str(infile),str(outfile),'--no-vis'],
                               capture_output=True,text=True,timeout=30,cwd=folder)
        validator_returncode=checked.returncode
        (model_dir/f'{seed:04d}.validator.stdout.txt').write_text(checked.stdout,encoding='utf-8')
        (model_dir/f'{seed:04d}.validator.stderr.txt').write_text(checked.stderr,encoding='utf-8')
        match=re.fullmatch(r'\s*Score\s*=\s*(\d+)\s*',checked.stdout)
        if checked.returncode==0 and match:
            score=int(match[1]);penalty=score-total_ops
            if penalty>=0 and penalty%100000==0:remaining=penalty//100000;valid=True
    stderr=stderr_file.read_text(encoding='utf-8',errors='replace')
    logs={};tail_phases=[]
    for line in stderr.splitlines():
        if ': ' not in line:continue
        prefix=line.split(': ',1)[0]
        if prefix=='TAIL_PHASE':tail_phases.append(fields(line))
        elif prefix in ['TIME','STAT','CHECKPOINT','TAIL','LOCAL_SEARCH','LOCAL_RESULT','WINDOW_CACHE','REV_SEARCH']:
            logs[prefix]=fields(line)
    raw=infile.read_text(encoding='utf-8').split()
    slimes=sum(c.islower() for line in raw[2:] for c in line)
    walls=sum(c=='#' for line in raw[2:] for c in line)
    branch='A44' if slimes<=120 or (slimes<=145 and walls<100) else 'B01'
    checkpoint=logs.get('CHECKPOINT',{}).get('ops')
    tail_before=logs.get('TAIL',{}).get('before')
    tail_after=logs.get('TAIL',{}).get('after')
    if name=='C_02' and branch=='B01':checkpoint=total_ops
    monotonic=None
    if name=='C_02':
        monotonic=(checkpoint is not None and total_ops<=checkpoint)
        if branch=='A44':monotonic=monotonic and tail_before==checkpoint and tail_after==total_ops
    return dict(seed=f'{seed:04d}',model=name,branch=branch,source_hash=hashes[name],input_hash=sha256(infile),
                valid=valid,remaining=remaining,score=score,total_ops=total_ops,elapsed_ms=elapsed_ms,
                over_1800ms=elapsed_ms>1800,over_2s=elapsed_ms>=2000,timed_out=timeout,returncode=returncode,
                validator_returncode=validator_returncode,checkpoint_ops=checkpoint,
                monotonic=monotonic,tail_saved_ops=checkpoint-total_ops if checkpoint is not None else None,
                sa_iters=logs.get('STAT',{}).get('sa_iters'),native_elapsed_ms=logs.get('TIME',{}).get('elapsed_ms'),
                logs=logs,tail_phases=tail_phases)


def aggregate(rows):
    summary={}
    for name in ['C_01','C_02']:
        subset=[r for r in rows if r['model']==name]
        successful=all(r['valid'] and r['remaining']==0 for r in subset)
        summary[name]=dict(cases=len(subset),total_ops=sum(r['total_ops'] for r in subset) if successful else None,
            invalid=sum(not r['valid'] for r in subset),incomplete=sum(r['valid'] and r['remaining']!=0 for r in subset),
            panic_or_exit_failure=sum(r['returncode'] not in [0,None] for r in subset),timeouts=sum(r['timed_out'] for r in subset),
            over_1800ms=sum(r['over_1800ms'] for r in subset),over_2s=sum(r['over_2s'] for r in subset),
            mean_elapsed_ms=sum(r['elapsed_ms'] for r in subset)/len(subset),max_elapsed_ms=max(r['elapsed_ms'] for r in subset))
    new=[r for r in rows if r['model']=='C_02']
    summary['tail']=dict(checkpoint_ops=sum(r['checkpoint_ops'] for r in new) if all(r['checkpoint_ops'] is not None for r in new) else None,
        final_ops=summary['C_02']['total_ops'],saved_ops=sum(r['tail_saved_ops'] or 0 for r in new),
        improved_cases=sum((r['tail_saved_ops'] or 0)>0 for r in new),monotonic_failures=sum(r['monotonic'] is not True for r in new),
        elapsed_ms=sum(r['logs'].get('TAIL',{}).get('elapsed_ms',0) for r in new),phases={},search={},results={},cache={},stops={})
    tail=summary['tail']
    for r in new:
        reason=r['logs'].get('TAIL',{}).get('stop_reason','B01_disabled');tail['stops'][reason]=tail['stops'].get(reason,0)+1
        for phase in r['tail_phases']:
            p=tail['phases'].setdefault(phase['name'],dict(trials=0,accepted=0,saved_ops=0,elapsed_ms=0))
            for key in p:p[key]+=phase.get(key,0)
        for label,prefix in [('search','LOCAL_SEARCH'),('results','LOCAL_RESULT'),('cache','WINDOW_CACHE')]:
            for key,value in r['logs'].get(prefix,{}).items():
                if isinstance(value,(int,float)):tail[label][key]=tail[label].get(key,0)+value
    pairs=[{r['model']:r for r in rows[i:i+2]} for i in range(0,len(rows),2)]
    valid_pairs=[p for p in pairs if all(r['valid'] and r['remaining']==0 for r in p.values())]
    diffs=[p['C_02']['total_ops']-p['C_01']['total_ops'] for p in valid_pairs]
    summary['comparison']=dict(valid_pairs=len(diffs),wins=sum(d<0 for d in diffs),ties=sum(d==0 for d in diffs),losses=sum(d>0 for d in diffs))
    for group in ['A44','B01','excluding_seed0']:
        subset=[p for p in valid_pairs if (p['C_02']['seed']!='0000' if group=='excluding_seed0' else p['C_02']['branch']==group)]
        summary['comparison'][group]=dict(cases=len(subset),C_01=sum(p['C_01']['total_ops'] for p in subset),C_02=sum(p['C_02']['total_ops'] for p in subset))
    return summary


def write_report(folder,rows,summary,metadata):
    (folder/'results.json').write_text(json.dumps(dict(metadata=metadata,summary=summary,cases=rows),indent=2),encoding='utf-8')
    columns=['seed','model','branch','source_hash','input_hash','valid','remaining','score','total_ops','checkpoint_ops','tail_saved_ops','monotonic','elapsed_ms','native_elapsed_ms','over_1800ms','over_2s','timed_out','returncode','validator_returncode','sa_iters']
    with (folder/'results.csv').open('w',encoding='utf-8',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=columns,extrasaction='ignore');writer.writeheader();writer.writerows(rows)
    old,new=summary['C_01'],summary['C_02'];compare=summary['comparison'];tail=summary['tail']
    text=['# C_02 / C_01 100ケース比較','',f"設定: `--tail={metadata['tail_mode']}`。0000～0099各1回。実行前に固定した設定のみを評価。",
          '1プロセスずつ実行し、偶数seedはC_01→C_02、奇数seedはC_02→C_01。両者とも `rustc --edition=2021 -O`。',
          '保存済み入力のハッシュを照合してコピーし、公式vis.exeで全出力を検証。失敗を0点として合計しない。','',
          '| 指標 | C_01 | C_02 |','|---|---:|---:|',
          f"| 合計手数 | {old['total_ops']} | {new['total_ops']} |",
          f"| 不正 | {old['invalid']} | {new['invalid']} |",f"| 未帰巣あり | {old['incomplete']} | {new['incomplete']} |",
          f"| panic・異常終了 | {old['panic_or_exit_failure']} | {new['panic_or_exit_failure']} |",
          f"| 1,800ms超 | {old['over_1800ms']} | {new['over_1800ms']} |",f"| 2秒以上 | {old['over_2s']} | {new['over_2s']} |",
          f"| 3秒打切り | {old['timeouts']} | {new['timeouts']} |",
          f"| 平均実行時間ms | {old['mean_elapsed_ms']:.3f} | {new['mean_elapsed_ms']:.3f} |",
          f"| 最大実行時間ms | {old['max_elapsed_ms']:.3f} | {new['max_elapsed_ms']:.3f} |",'']
    if old['total_ops'] is not None and new['total_ops'] is not None:
        diff=new['total_ops']-old['total_ops'];text += [f"同条件の差: {diff:+d}手 ({diff/old['total_ops']*100:+.3f}%)。",f"歴史的C_01の31,312手との差: {new['total_ops']-31312:+d}手。",'']
    text += [f"C_02の勝ち / 同点 / 負け: {compare['wins']} / {compare['ties']} / {compare['losses']}。",'',
             '| 内訳 | 件数 | C_01 | C_02 | 差 |','|---|---:|---:|---:|---:|']
    for group in ['A44','B01','excluding_seed0']:
        g=compare[group];text.append(f"| {group} | {g['cases']} | {g['C_01']} | {g['C_02']} | {g['C_02']-g['C_01']:+d} |")
    text += ['','## 同一実行内のtail効果','',f"checkpoint合計: {tail['checkpoint_ops']}。最終合計: {tail['final_ops']}。tailによる削減: {tail['saved_ops']}手。",
             f"tailで改善したケース: {tail['improved_cases']}件。final <= checkpointの違反: {tail['monotonic_failures']}件。",
             'B01の6件はtailを呼ばないため、その最終手数をcheckpoint集計に含める。','',
             '| tail処理 | 試行数 | 採用数 | 削減手数 | 時間ms |','|---|---:|---:|---:|---:|']
    for name,p in tail['phases'].items():text.append(f"| {name} | {p['trials']} | {p['accepted']} | {p['saved_ops']} | {p['elapsed_ms']:.3f} |")
    text += ['','localの時間はセッション全体（after_local整理を含む）であり、時間欄は単純合計できない。削減手数は処理間で二重計上しない。',
             'checkpointからの差は同一実行内でtailが追加した改善で、SAの反復数の揺れと切り分けられる。別実行のC_01との差全体がtailだけに由来するとは限らない。','',
             '## 探索・キャッシュ・終了理由','', '```json',json.dumps({k:tail[k] for k in ['search','results','cache','stops']},indent=2),'```','',
             '今回実測したのは上記の1設定だけ。legacy/offの性能比較と任意拡張Rは未実施。',
             '探索打切りは到達不能の証明として保存しない。完了した深さのみを失敗キャッシュへ記録する。',
             '実行時間はプロセス起動・終了を含むローカル壁時計時間。提出先の実行時間を保証する測定ではない。',
             '評価結果に基づく解答コードの追加改良や自動的な再評価は行っていない。','',
             '## 再現情報','', '```json',json.dumps(metadata,indent=2),'```','',
             '標準出力、stderr、公式検証器の標準出力・stderrはモデルごとのフォルダ。ケース別指標はresults.csv/results.json。']
    (folder/'REPORT.md').write_text('\n'.join(text)+'\n',encoding='utf-8')


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--tail',choices=['off','legacy','compact-fast'],default='compact-fast')
    parser.add_argument('--inputs',type=Path,default=OLD_RUN/'inputs')
    parser.add_argument('--vis',type=Path,default=OLD_RUN/'binaries/vis.exe')
    parser.add_argument('--run-dir',type=Path,default=ROOT/'outputs/C_02_comparison_20261003')
    args=parser.parse_args();folder=args.run_dir.resolve()
    if folder.exists():raise RuntimeError('Run folder exists; choose a fresh --run-dir to preserve results')
    if sha256(ROOT/'C_01.rs')!=C01_HASH:raise RuntimeError('Baseline C_01 hash differs from specification')
    historical=json.loads((OLD_RUN/'results.json').read_text(encoding='utf-8'))
    expected_inputs={r['seed']:r['input_hash'] for r in historical['cases']}
    for seed in range(100):
        path=args.inputs/f'{seed:04d}.txt'
        if sha256(path)!=expected_inputs[f'{seed:04d}']:raise RuntimeError(f'Input hash mismatch: {seed:04d}')
    folder.mkdir(parents=True)
    for name in ['binaries','inputs','C_01','C_02']:(folder/name).mkdir()
    for seed in range(100):shutil.copy2(args.inputs/f'{seed:04d}.txt',folder/'inputs'/f'{seed:04d}.txt')
    shutil.copy2(args.vis,folder/'binaries/vis.exe')
    compiler=subprocess.run(['rustc','--version'],capture_output=True,text=True,check=True).stdout.strip()
    hashes={name:compile_source(name,folder) for name in ['C_01','C_02']}
    metadata=dict(date='2026-10-03',tail_mode=args.tail,compiler=compiler,options=['--edition=2021','-O'],source_hashes=hashes,
                  validator_hash=sha256(folder/'binaries/vis.exe'),seeds='0000..0099',repetitions=1)
    (folder/'metadata.json').write_text(json.dumps(metadata,indent=2),encoding='utf-8')
    rows=[];start=time.perf_counter()
    for seed in range(100):
        order=['C_01','C_02'] if seed%2==0 else ['C_02','C_01']
        for name in order:rows.append(run_case(name,seed,folder,hashes,args.tail))
        (folder/'progress.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
        if (seed+1)%10==0:print(f'{seed+1}/100 pairs completed; elapsed={time.perf_counter()-start:.1f}s',flush=True)
    summary=aggregate(rows);write_report(folder,rows,summary,metadata)
    print(json.dumps(summary,indent=2),flush=True);print(f'Report: {folder / "REPORT.md"}',flush=True)
    return int(any(not r['valid'] or r['remaining']!=0 or r['over_2s'] or (r['model']=='C_02' and r['monotonic'] is not True) for r in rows))


if __name__=='__main__':sys.exit(main())
