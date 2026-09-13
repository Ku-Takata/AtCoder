import os
import sys
import subprocess
import time
from concurrent.futures import ProcessPoolExecutor

def analyze_seed(args):
    seed, target_exe, tester_exe, in_dir = args
    seed_str = f"{seed:04d}"
    infile = os.path.join(in_dir, f"{seed_str}.txt")
    
    try:
        with open(infile, "r") as f:
            p = subprocess.run(
                [tester_exe, target_exe],
                stdin=f,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=15
            )
        
        stderr = p.stderr
        exec_time = 0.0
        guard_warning = False
        phase3_cnt = 0
        mm_cnt = 0
        score = 0
        
        for line in stderr.splitlines():
            if "Execution Time:" in line:
                parts = line.split(":")
                if len(parts) >= 2:
                    t_str = parts[1].strip().split(" ")[0]
                    try:
                        exec_time = float(t_str)
                    except:
                        pass
            if "High Execution Time" in line:
                guard_warning = True
            if "Phase3=" in line:
                for p_part in line.split("|"):
                    if "Phase3=" in p_part:
                        try:
                            phase3_cnt = int(p_part.strip().split("=")[1])
                        except:
                            pass
            if "Moves:" in line:
                try:
                    mm_cnt = int(line.split(":")[1].split("(")[0].strip())
                except:
                    pass
            if "Score = " in line:
                try:
                    score = int(line.strip().split("= ")[1])
                except:
                    pass
                
        return seed, score, exec_time, guard_warning, phase3_cnt, mm_cnt
    except Exception as e:
        return seed, 0, 0.0, False, 0, 0

def main():
    target_exe = sys.argv[1] if len(sys.argv) > 1 else "./main_rs.exe"
    tester_exe = "AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/tester.exe"
    in_dir = "AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/in"
    num_seeds = 100

    print(f"==================================================")
    print(f" Checking Time Cutoffs for: {target_exe}")
    print(f"==================================================", flush=True)

    tasks = [(seed, target_exe, tester_exe, in_dir) for seed in range(num_seeds)]
    
    results = []
    with ProcessPoolExecutor() as executor:
        for res in executor.map(analyze_seed, tasks):
            results.append(res)
            
    results.sort(key=lambda x: x[0])
    
    times = [r[2] for r in results]
    scores = [r[1] for r in results]
    warnings = [r for r in results if r[3]]
    over_08 = [r for r in results if r[2] >= 0.80]
    over_14 = [r for r in results if r[2] >= 1.40]
    over_16 = [r for r in results if r[2] >= 1.60]

    print("\n--------------------------------------------------")
    print(f" Tested Cases:           {len(results)}")
    print(f" Avg Score:              {sum(scores)/len(scores):,.2f}")
    print(f" Avg Execution Time:     {sum(times)/len(times):.3f} s")
    print(f" Max Execution Time:     {max(times):.3f} s (Seed {max(results, key=lambda x: x[2])[0]:04d})")
    print(f" Min Execution Time:     {min(times):.3f} s (Seed {min(results, key=lambda x: x[2])[0]:04d})")
    print("--------------------------------------------------")
    print(f" Cases >= 0.80s (Phase3 trial limit 96 -> 48): {len(over_08)} / {num_seeds}")
    print(f" Cases >= 1.40s (Phase3 trial limit 48 -> 24): {len(over_14)} / {num_seeds}")
    print(f" Cases >= 1.60s (MultiMove Skips / Cutoff):    {len(over_16)} / {num_seeds}")
    print(f" Safety Guard Warnings (>1.5s):                 {len(warnings)} / {num_seeds}")
    print("--------------------------------------------------")

    if over_16:
        print("\n--- Cases hitting 1.60s MultiMove Cutoff ---")
        for seed, score, exec_time, _, p3, mm in over_16:
            print(f"Seed {seed:04d}: Score={score:,}, Time={exec_time:.3f}s, Phase3={p3}, Moves={mm}")
            
    print(f"==================================================\n")

if __name__ == "__main__":
    main()
