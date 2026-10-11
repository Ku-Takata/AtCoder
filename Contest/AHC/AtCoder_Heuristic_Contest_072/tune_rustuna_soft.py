import os
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
import rustuna

in_dir = r"C:\Users\kuon9\Downloads\BrsBAXF2\tools\in"
exe_path = r".\B_03_tune.exe"

# Balanced representative 30 seeds across all tiers
# Small (<=75): 16 seeds
# Medium (76-115): 8 seeds
# Dense (>115): 6 seeds
tune_seeds = [
    # Small
    0, 1, 6, 12, 19, 23, 27, 33, 40, 43, 50, 56, 62, 69, 74, 88,
    # Medium
    4, 17, 31, 45, 49, 58, 66, 73,
    # Dense
    5, 14, 29, 71, 81, 97
]

def eval_single(seed_int, params_args):
    seed = f"{seed_int:04d}"
    in_file = os.path.join(in_dir, f"{seed}.txt")

    cmd = [exe_path] + params_args
    with open(in_file, "r") as fin:
        proc = subprocess.run(cmd, stdin=fin, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

    score = 10000000
    for line in proc.stderr.splitlines():
        if "STAT:" in line:
            for part in line.split():
                if part.startswith("final_score="):
                    score = int(part.split("=")[1])
                    break
    return score

def objective(trial):
    # 1. 巣穴射出（案C）のソフト費用便益モデル
    launch_c_base = trial.suggest_float("launch_c_base", 0.1, 0.8, step=0.05)
    launch_c_rho = trial.suggest_float("launch_c_rho", 0.2, 2.0, step=0.1)
    launch_c_sq = trial.suggest_float("launch_c_sq", 0.5, 3.0, step=0.25)
    launch_b_min = trial.suggest_float("launch_b_min", -0.8, 0.2, step=0.1)

    # 2. 事前クラスタリング（案B）のソフト距離・ゲインモデル
    clust_dist_base = trial.suggest_float("clust_dist_base", 5.0, 8.0, step=0.5)
    clust_dist_slope = trial.suggest_float("clust_dist_slope", 4.0, 10.0, step=1.0)
    clust_w_dist = trial.suggest_float("clust_w_dist", 1.0, 3.5, step=0.5)

    # 3. 樹状合流（案A）のソフトペナルティモデル
    tree_pen_base = trial.suggest_float("tree_pen_base", 0.5, 3.0, step=0.5)
    tree_pen_slope = trial.suggest_float("tree_pen_slope", 1.0, 6.0, step=1.0)
    tree_bon_fwd = trial.suggest_float("tree_bon_fwd", 0.0, 2.0, step=0.5)
    x_w_base = trial.suggest_float("x_w_base", 0.0, 0.5, step=0.1)
    x_w_rho = trial.suggest_float("x_w_rho", 0.0, 0.8, step=0.1)

    # 4. SA 温度
    t_start = trial.suggest_float("t_start", 6.0, 14.0, step=2.0)
    t_end = trial.suggest_float("t_end", 0.1, 0.8, step=0.1)

    params_args = [
        "--launch_c_base", f"{launch_c_base:.2f}",
        "--launch_c_rho", f"{launch_c_rho:.2f}",
        "--launch_c_sq", f"{launch_c_sq:.2f}",
        "--launch_b_min", f"{launch_b_min:.2f}",

        "--clust_dist_base", f"{clust_dist_base:.2f}",
        "--clust_dist_slope", f"{clust_dist_slope:.2f}",
        "--clust_w_dist", f"{clust_w_dist:.2f}",

        "--tree_pen_base", f"{tree_pen_base:.2f}",
        "--tree_pen_slope", f"{tree_pen_slope:.2f}",
        "--tree_bon_fwd", f"{tree_bon_fwd:.2f}",
        "--x_w_base", f"{x_w_base:.2f}",
        "--x_w_rho", f"{x_w_rho:.2f}",

        "--t_start", f"{t_start:.1f}",
        "--t_end", f"{t_end:.1f}",
    ]

    t0 = time.time()
    with ThreadPoolExecutor(max_workers=8) as executor:
        scores = list(executor.map(lambda s: eval_single(s, params_args), tune_seeds))
    iter_elapsed = time.time() - t0

    total_score = sum(scores)
    cleared_count = sum(1 for s in scores if s < 100000)

    print(
        f"[Trial {trial.number:03d}] Score: {total_score:,} ({iter_elapsed:.1f}s) | "
        f"launch: c_b={launch_c_base:.2f} c_r={launch_c_rho:.2f} sq={launch_c_sq:.2f} b_m={launch_b_min:.2f} | "
        f"clust: d_b={clust_dist_base:.1f} d_s={clust_dist_slope:.1f} w_d={clust_w_dist:.1f} | "
        f"tree: p_b={tree_pen_base:.1f} p_s={tree_pen_slope:.1f} b_f={tree_bon_fwd:.1f} | "
        f"t_s={t_start:.1f} t_e={t_end:.1f}",
        flush=True,
    )
    return total_score

def main():
    print(f"=== Starting Rustuna Tuning for Soft Parameters (30 Balanced Seeds) ===", flush=True)
    start_t = time.time()
    study = rustuna.create_study(direction="minimize")

    # Baseline configuration
    baseline = {
        "launch_c_base": 0.3,
        "launch_c_rho": 0.8,
        "launch_c_sq": 1.5,
        "launch_b_min": -0.5,

        "clust_dist_base": 6.5,
        "clust_dist_slope": 8.0,
        "clust_w_dist": 2.0,

        "tree_pen_base": 1.5,
        "tree_pen_slope": 4.0,
        "tree_bon_fwd": 1.0,
        "x_w_base": 0.2,
        "x_w_rho": 0.5,

        "t_start": 10.0,
        "t_end": 0.3,
    }
    study.enqueue_trial(baseline)

    study.optimize(objective, n_trials=35)
    elapsed = time.time() - start_t

    best = study.best_trial
    print("\n" + "="*60, flush=True)
    print("=== Tuning Finished ===", flush=True)
    print(f"Elapsed: {elapsed:.2f}s ({elapsed/60:.1f}min)", flush=True)
    print(f"Best Total Score (30 seeds): {best.value:,}", flush=True)
    print("Best Parameters:", flush=True)
    for k, v in sorted(best.params.items()):
        print(f"  {k} = {v}", flush=True)

    # Save best parameters to a python file for benchmark
    with open("best_tuned_params.py", "w") as f:
        f.write("# Automatically generated by Rustuna tuning\n")
        f.write("BEST_PARAMS = {\n")
        for k, v in sorted(best.params.items()):
            f.write(f"    '{k}': {v},\n")
        f.write("}\n")
    print("Saved best parameters to best_tuned_params.py", flush=True)

if __name__ == "__main__":
    main()
