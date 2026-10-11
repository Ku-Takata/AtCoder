import subprocess
import os
import time

exe = 'B_06.exe'
in_dir = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\in'
out_dir = r'outputs\B_06'
vis_exe = r'C:\Users\kuon9\Downloads\BrsBAXF2\tools\target\release\vis.exe'

def main():
    total_score = 0
    scores = []
    for i in range(100):
        seed = f'{i:04d}'
        in_file = os.path.join(in_dir, f'{seed}.txt')
        out_file = os.path.join(out_dir, f'{seed}.txt')
        vis_proc = subprocess.run([vis_exe, in_file, out_file], capture_output=True, text=True)
        score = 0
        for line in vis_proc.stdout.splitlines():
            if 'Score =' in line:
                score = int(line.split('=')[1].strip())
        scores.append(score)
        total_score += score
    print(f"B_06 Verified 100-seed Total Score: {total_score:,} moves")
    assert total_score == 32193, f"Expected 32,193 but got {total_score}"

if __name__ == '__main__':
    main()
