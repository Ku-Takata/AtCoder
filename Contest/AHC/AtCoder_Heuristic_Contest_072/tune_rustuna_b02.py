import os
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
import rustuna

in_dir = r"C:\Users\kuon9\Downloads\BrsBAXF2\tools\in"
exe_path = r".\B_02.exe"

tune_seeds = list(range(100))

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
                elif part.startswith("best_score="):
                    score = int(part.split("=")[1])
                    break
    return score

def objective(trial):
    th_small = trial.suggest_int("th_small", 70, 100, step=5)
    th_mid = trial.suggest_int("th_mid", 105, 125, step=5)
    t_start = trial.suggest_float("t_start", 4.0, 16.0, step=2.0)
    t_end = trial.suggest_float("t_end", 0.1, 1.5, step=0.2)
    w_strat_tree = trial.suggest_int("w_strat_tree", 1, 5)
    w_strat_sweep = trial.suggest_int("w_strat_sweep", 1, 5)
    w_strat_catapult = trial.suggest_int("w_strat_catapult", 0, 3)
    w_strat_centroid = trial.suggest_int("w_strat_centroid", 0, 2)
    p_strategy = trial.suggest_int("p_strategy", 5, 25, step=5)

    params_args = [
        "--th_small", str(th_small),
        "--th_mid", str(th_mid),
        "--t_start", f"{t_start:.1f}",
        "--t_end", f"{t_end:.1f}",
        "--w_strat_tree", str(w_strat_tree),
        "--w_strat_sweep", str(w_strat_sweep),
        "--w_strat_catapult", str(w_strat_catapult),
        "--w_strat_centroid", str(w_strat_centroid),
        "--p_strategy", str(p_strategy),
    ]

    t0 = time.time()
    with ThreadPoolExecutor(max_workers=8) as executor:
        scores = list(executor.map(lambda s: eval_single(s, params_args), tune_seeds))
    iter_elapsed = time.time() - t0

    total_score = sum(scores)
    cleared_count = sum(1 for s in scores if s < 100000)
    cleared_scores = [s for s in scores if s < 100000]
    avg_cleared = sum(cleared_scores) / len(cleared_scores) if cleared_scores else 0

    print(
        f"[Trial {trial.number:03d}] Score: {total_score:,} | Cleared: {cleared_count:3d}/100 | Avg: {avg_cleared:.2f} ({iter_elapsed:.1f}s) | "
        f"th_s={th_small} th_m={th_mid} t_s={t_start:.1f} t_e={t_end:.1f} "
        f"tree={w_strat_tree} swp={w_strat_sweep} cat={w_strat_catapult} cnt={w_strat_centroid} p_st={p_strategy}",
        flush=True,
    )
    return total_score

if __name__ == "__main__":
    print(f"=== Starting Rustuna Tuning for B_02 (100 SEEDS FULL EVALUATION) ===", flush=True)
    start_t = time.time()
    study = rustuna.create_study(direction="minimize")

    # Baseline configuration (B_02 default: 36,347)
    baseline = {
        "th_small": 75,
        "th_mid": 115,
        "t_start": 8.0,
        "t_end": 0.5,
        "w_strat_tree": 2,
        "w_strat_sweep": 2,
        "w_strat_catapult": 1,
        "w_strat_centroid": 1,
        "p_strategy": 13,
    }
    study.enqueue_trial(baseline)

    study.optimize(objective, n_trials=35)
    elapsed = time.time() - start_t

    best = study.best_trial
    print("\n=== Tuning Finished ===")
    print(f"Elapsed: {elapsed:.2f}s ({elapsed/60:.1f}min)")
    print(f"Best Total Score: {best.value:,}")
    print("Best Parameters:")
    for k, v in sorted(best.params.items()):
        print(f"  {k} = {v}")
