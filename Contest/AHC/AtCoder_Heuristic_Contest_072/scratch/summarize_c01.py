"""Read benchmark evidence and append analysis; does not execute or change solvers."""
import json
import re
from pathlib import Path

root=Path(__file__).resolve().parents[1]
folder=root/'outputs/C_01_comparison_20261003'
data=json.loads((folder/'results.json').read_text(encoding='utf-8'))
aggregates={}
phase_durations={}
for row in data['cases']:
    if row['model']!='C_01':continue
    for line in row['phases']:
        name=re.search(r'name=(\S+)',line)
        if not name:continue
        name=name[1]
        if line.startswith('PHASE_STATS:'):
            current=aggregates.setdefault(name,{})
            for key,value in re.findall(r'(\w+)=(\d+)',line):
                current[key]=current.get(key,0)+int(value)
        elif line.startswith('PHASE:'):
            duration=re.search(r'elapsed_ms=([\d.]+)',line)
            if duration:phase_durations[name]=phase_durations.get(name,0)+float(duration[1])

analysis=['', '## 新しい近傍の探索記録', '',
          '| 処理 | 候補検証数 | 採用数 | 削減手数合計 | 局所展開ノード数 | 合計処理時間ms |',
          '|---|---:|---:|---:|---:|---:|']
for name in ['bridge','compact']:
    s=aggregates.get(name,{})
    analysis.append(f"| {name} | {s.get('trials',0)} | {s.get('accepted',0)} | {s.get('saved_ops',0)} | {s.get('compact_nodes',0)} | {phase_durations.get(name,0):.3f} |")
analysis += ['',
    '局所探索は終了状態の完全一致と区間全体の再生検証で採用するため、compactの候補検証数は共通validate_patchのtrialsには加算されない。',
    'これらの削減手数は処理した全候補解についての累積であり、最終出力だけの削減手数ではない。共通PATCH統計も同じ集計範囲。',
    '新しい近傍が実際に短い置換を発見したことはログで確認できる。一方、今回の対応比較は1回で、fast/bridge/localの分離評価はしていないため、合計948手の差を各変更や時間の揺れに厳密に割り当てることはできない。', '',
    '## 評価中断と再開', '',
    (folder/'interruption.txt').read_text(encoding='utf-8').strip(), '',
    '再開時に完了済み6ペアの入力・ソースハッシュを照合。解答実行ファイルは再コンパイルせず、未完了の0006ペアから再開した。以後はコピー済みの入力と公式検証器を参照した。']
with (folder/'REPORT.md').open('a',encoding='utf-8') as f:f.write('\n'.join(analysis)+'\n')
print(json.dumps(dict(phases=aggregates,durations_ms=phase_durations),indent=2))
