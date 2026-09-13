import optuna
import subprocess
import concurrent.futures
import re
import os
import glob
import sys

EXEC_PATH = "./main15.exe"
TESTER_PATH = r"AdcJXWH4_windows\tools_x86_64-pc-windows-gnu\tester.exe"
IN_DIR = r"AdcJXWH4_windows\tools_x86_64-pc-windows-gnu\in" 

NUM_CASES = 1000
N_TRIALS_PER_RUN = 50

fn_score = re.compile(r"Score\s*=\s*(\d+)")

def run_case(seed_file, pond_count_th, pond_cluster_th, p2_th_relaxed, p3_relax_mult):
    cmd = [
        TESTER_PATH,
        EXEC_PATH,
        # (1~12) 固定パラメータ
        "0.000930648811439626", "0.0015812296573779147", "0.004643700270792169", "0.0008822059737903149",
        "0.0019684143784392803", "0.0010232407503122955", "0.003947946318465455", "0.0006210630252290683",
        "1.8232673356521643", "1.5458212320758866", "1.4074403758571954", "514.1966045939967",
        # (13~20) MultiMove固定
        "0.85", "300000.0", "0.90", "1.5", "2.0", "1000000.0", "3", "4",
        # (21~25) 固定
        "1.04", "7498.415", "1.30", "0.40", "3",
        # (26~27) 池判定パラメータ (Optunaチューニング対象)
        str(pond_count_th),
        str(pond_cluster_th),
        # (28~29) 固定
        "0.0", "0.05",
        # (30~31) 池緩和パラメータ (Optunaチューニング対象)
        str(p2_th_relaxed),
        str(p3_relax_mult),
    ]
    
    with open(seed_file, "r", encoding="utf-8") as f:
        res = subprocess.run(cmd, stdin=f, capture_output=True, text=True, encoding="utf-8")

    combined = (res.stdout or "") + "\n" + (res.stderr or "")
    match = fn_score.search(combined)
    return int(match.group(1)) if match else 0

def run_cases_parallel(seed_files, pond_count_th, pond_cluster_th, p2_th_relaxed, p3_relax_mult):
    total_score = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=12) as executor:
        futures = [executor.submit(run_case, f, pond_count_th, pond_cluster_th, p2_th_relaxed, p3_relax_mult) for f in seed_files]
        for future in concurrent.futures.as_completed(futures):
            total_score += future.result()
    return total_score

def objective(trial):
    # ユーザー指定：池の数・ばらつき判定と緩和定数のみに特化したOptuna探索空間
    pond_count_th = trial.suggest_float("pond_count_th", 300.0, 900.0, step=10.0)
    pond_cluster_th = trial.suggest_float("pond_cluster_th", 1.20, 2.50, step=0.05)
    p2_th_relaxed = trial.suggest_float("p2_th_relaxed", 0.65, 0.82, step=0.01)
    p3_relax_mult = trial.suggest_float("p3_relax_mult", 1.10, 2.00, step=0.05)

    all_seed_files = sorted(glob.glob(os.path.join(IN_DIR, "*.txt")))[:NUM_CASES]
    total_score = run_cases_parallel(all_seed_files, pond_count_th, pond_cluster_th, p2_th_relaxed, p3_relax_mult)
    return total_score

if __name__ == "__main__":
    if not os.path.exists(EXEC_PATH):
        print(f"【エラー】実行ファイル '{EXEC_PATH}' が見つかりません。")
        sys.exit(1)

    db_name = "sqlite:///ahc069_optuna.db"
    study_name = "ahc069_tuning_pond_relaxation_only"

    study = optuna.create_study(
        study_name=study_name,
        storage=db_name,
        load_if_exists=True,
        direction="maximize"
    )

    completed_trials = [t for t in study.trials if t.state == optuna.trial.TrialState.COMPLETE]
    print(f"=== {EXEC_PATH} (池散在判定＆緩和定数 特化チューナー) ===")
    print(f"過去の蓄積完走トライアル数: {len(completed_trials)} 回")
    if len(completed_trials) > 0:
        print(f"現在の最高スコア (1000ケース合計): {study.best_value:,.0f} (平均: {study.best_value / 1000.0:,.1f})")
        print(f"  最適 pond_count_th: {study.best_trial.params['pond_count_th']:.1f}")
        print(f"  最適 pond_cluster_th: {study.best_trial.params['pond_cluster_th']:.2f}")
        print(f"  最適 p2_th_relaxed: {study.best_trial.params['p2_th_relaxed']:.2f}")
        print(f"  最適 p3_relax_mult: {study.best_trial.params['p3_relax_mult']:.2f}")
    
    print(f"\n今回のセッションで +{N_TRIALS_PER_RUN} 回の試行を追加実行します...")
    study.optimize(objective, n_trials=N_TRIALS_PER_RUN)

    completed_trials_end = [t for t in study.trials if t.state == optuna.trial.TrialState.COMPLETE]
    print("\n========================================")
    print(f"最適化セッション完了！ (通算 {len(completed_trials_end)} トライアル完走)")
    if len(completed_trials_end) > 0:
        print(f"  最高スコア (1000ケース合計): {study.best_value:,.0f}")
        print(f"  1000ケース平均スコア: {study.best_value / 1000.0:,.1f}")
        print(f"  最適 pond_count_th: {study.best_trial.params['pond_count_th']:.1f}")
        print(f"  最適 pond_cluster_th: {study.best_trial.params['pond_cluster_th']:.2f}")
        print(f"  最適 p2_th_relaxed: {study.best_trial.params['p2_th_relaxed']:.2f}")
        print(f"  最適 p3_relax_mult: {study.best_trial.params['p3_relax_mult']:.2f}")
