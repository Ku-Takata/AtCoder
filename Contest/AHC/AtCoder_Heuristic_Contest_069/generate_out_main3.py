import os
import subprocess

os.makedirs("out_main3", exist_ok=True)

print("Generating 100 output files for main3.exe into 'out_main3/'...")
for seed in range(100):
    seed_str = f"{seed:04d}"
    infile = f"AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/in/{seed_str}.txt"
    outfile = f"out_main3/{seed_str}.txt"
    tester = "AdcJXWH4_windows/tools_x86_64-pc-windows-gnu/tester.exe"
    
    with open(infile, "r") as fin, open(outfile, "w") as fout:
        # Save stdout of tester + main3 to outfile so user can view with visualizer
        p = subprocess.run([tester, "./main3.exe"], stdin=fin, stdout=fout, stderr=subprocess.PIPE, text=True)
    
    # Extract score for verification
    score = "N/A"
    for line in p.stderr.split('\n'):
        if 'Score = ' in line:
            score = line.strip().split('= ')[1]
            break
    if seed % 10 == 0:
        print(f"Seed {seed_str}: Score = {score} -> Saved to {outfile}")

print("\nDone! All 100 output files saved in 'out_main3/' directory.")
