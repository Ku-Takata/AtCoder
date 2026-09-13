import os
import sys
import subprocess
from concurrent.futures import ProcessPoolExecutor

def generate_one(args):
    seed, target_exe, tester_exe, in_dir, out_dir = args
    seed_str = f"{seed:04d}"
    infile = os.path.join(in_dir, f"{seed_str}.txt")
    outfile = os.path.join(out_dir, f"{seed_str}.txt")
    
    with open(infile, "r") as fin, open(outfile, "w") as fout:
        subprocess.run([tester_exe, target_exe], stdin=fin, stdout=fout, stderr=subprocess.PIPE, text=True)
    return seed_str

def main():
    target_exe = sys.argv[1] if len(sys.argv) > 1 else "./main_rs.exe"
    out_dir = sys.argv[2] if len(sys.argv) > 2 else "outputs_rs"
    tester_exe = "AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/tester.exe"
    in_dir = "AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/in"

    # in_dir 内の .txt ファイル数から自動検出
    num_seeds = len([f for f in os.listdir(in_dir) if f.endswith(".txt")])

    os.makedirs(out_dir, exist_ok=True)
    print(f"Generating {num_seeds} output files for '{target_exe}' into '{out_dir}/'...")

    tasks = [(seed, target_exe, tester_exe, in_dir, out_dir) for seed in range(num_seeds)]
    
    with ProcessPoolExecutor() as executor:
        for seed_str in executor.map(generate_one, tasks):
            pass

    print(f"Done! All {num_seeds} output files saved into '{out_dir}/'.")

if __name__ == "__main__":
    main()