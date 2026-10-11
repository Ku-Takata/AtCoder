# C_02 / C_01 100ケース比較

設定: `--tail=compact-fast`。0000～0099各1回。実行前に固定した設定のみを評価。
1プロセスずつ実行し、偶数seedはC_01→C_02、奇数seedはC_02→C_01。両者とも `rustc --edition=2021 -O`。
保存済み入力のハッシュを照合してコピーし、公式vis.exeで全出力を検証。失敗を0点として合計しない。

| 指標 | C_01 | C_02 |
|---|---:|---:|
| 合計手数 | 31549 | 31374 |
| 不正 | 0 | 0 |
| 未帰巣あり | 0 | 0 |
| panic・異常終了 | 0 | 0 |
| 1,800ms超 | 0 | 1 |
| 2秒以上 | 0 | 0 |
| 3秒打切り | 0 | 0 |
| 平均実行時間ms | 1640.166 | 1676.354 |
| 最大実行時間ms | 1792.911 | 1808.425 |

同条件の差: -175手 (-0.555%)。
歴史的C_01の31,312手との差: +62手。

C_02の勝ち / 同点 / 負け: 51 / 30 / 19。

| 内訳 | 件数 | C_01 | C_02 | 差 |
|---|---:|---:|---:|---:|
| A44 | 94 | 26545 | 26374 | -171 |
| B01 | 6 | 5004 | 5000 | -4 |
| excluding_seed0 | 99 | 31505 | 31330 | -175 |

## 同一実行内のtail効果

checkpoint合計: 31532。最終合計: 31374。tailによる削減: 158手。
tailで改善したケース: 51件。final <= checkpointの違反: 0件。
B01の6件はtailを呼ばないため、その最終手数をcheckpoint集計に含める。

| tail処理 | 試行数 | 採用数 | 削減手数 | 時間ms |
|---|---:|---:|---:|---:|
| initial_geometry | 1032980 | 27 | 31 | 524.288 |
| local | 178097 | 118 | 121 | 2967.420 |
| after_local | 432218 | 5 | 6 | 225.691 |

localの時間はセッション全体（after_local整理を含む）であり、時間欄は単純合計できない。削減手数は処理間で二重計上しない。
checkpointからの差は同一実行内でtailが追加した改善で、SAの反復数の揺れと切り分けられる。別実行のC_01との差全体がtailだけに由来するとは限らない。

## 探索・キャッシュ・終了理由

```json
{
  "search": {
    "windows": 178097,
    "generated": 82725305,
    "calls": 8210136,
    "expanded": 3738339,
    "terminal_checks": 1871398,
    "first_half": 85142,
    "second_half": 92955,
    "reverse_windows": 114026,
    "forward_windows": 64071
  },
  "results": {
    "found": 118,
    "exhausted": 87486,
    "node_limit": 116,
    "time_limit": 16
  },
  "cache": {
    "hits_found": 0,
    "hits_exhausted": 90308,
    "transient_skips": 53,
    "entries": 87721,
    "elapsed_ms": 90.768
  },
  "stops": {
    "no_new_work": 71,
    "deadline": 23,
    "B01_disabled": 6
  }
}
```

今回実測したのは上記の1設定だけ。legacy/offの性能比較と任意拡張Rは未実施。
探索打切りは到達不能の証明として保存しない。完了した深さのみを失敗キャッシュへ記録する。
実行時間はプロセス起動・終了を含むローカル壁時計時間。提出先の実行時間を保証する測定ではない。
評価結果に基づく解答コードの追加改良や自動的な再評価は行っていない。

## 再現情報

```json
{
  "date": "2026-10-03",
  "tail_mode": "compact-fast",
  "compiler": "rustc 1.96.0 (ac68faa20 2026-05-25)",
  "options": [
    "--edition=2021",
    "-O"
  ],
  "source_hashes": {
    "C_01": "B5BDC5F64924C97E4AD37F4FB1788AAC5EAD35B6D1D464226832CF7E7384FD5A",
    "C_02": "C1052DB85C1E394AF1A5A06292B2A17FFDCA8FB0901A070DC7F4BC23963EF7C8"
  },
  "validator_hash": "B411112725F58099CA56FC16960A8F3A3DCF577A074430CB21865F918F4AA905",
  "seeds": "0000..0099",
  "repetitions": 1
}
```

標準出力、stderr、公式検証器の標準出力・stderrはモデルごとのフォルダ。ケース別指標はresults.csv/results.json。
