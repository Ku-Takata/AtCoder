import glob
import os
import subprocess
import sys

def load_input(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    lines = content.strip().split("\n")
    if len(lines) < 3:
        raise ValueError(f"Invalid input file: {file_path}")
    
    W, H, K = map(int, lines[0].split())
    c1, c3, c5, c7, c9 = map(int, lines[1].split())
    costs = {1: c1, 3: c3, 5: c5, 7: c7, 9: c9}
    
    holes = []
    for line in lines[2:2 + K]:
        if line.strip():
            a, b = map(int, line.split())
            holes.append((a, b))
            
    info = {
        "W": W,
        "H": H,
        "K": K,
        "costs": costs,
        "holes": holes
    }
    return content, info

def validate_and_score(output_str, info):
    lines = output_str.strip().split("\n")
    if not lines or not lines[0].strip():
        return False, "Empty output", 0, 0
    try:
        M = int(lines[0].strip())
    except ValueError:
        return False, "M is not an integer", 0, 0
    
    bricks = []
    for i in range(1, M + 1):
        if i >= len(lines):
            return False, f"Expected {M} bricks, but got {len(lines)-1}", 0, 0
        parts = list(map(int, lines[i].split()))
        if len(parts) != 3:
            return False, f"Line {i} format invalid: {lines[i]}", 0, 0
        x, y, l = parts
        bricks.append((x, y, l))

    # 条件1: 壁内
    W, H = info["W"], info["H"]
    for x, y, l in bricks:
        if l not in [1, 3, 5, 7, 9]:
            return False, f"Invalid length: {l}", 0, 0
        if not (0 <= x and x + l <= W and 0 <= y < H):
            return False, f"Out of bounds: ({x}, {y}, {l})", 0, 0

    # 条件2: 重複なし
    grid = {}
    for idx, (x, y, l) in enumerate(bricks):
        for dx in range(l):
            pos = (x + dx, y)
            if pos in grid:
                return False, f"Overlap at {pos} between bricks", 0, 0
            grid[pos] = idx

    # 条件3: 中央で支持
    for idx, (x, y, l) in enumerate(bricks):
        if y == 0:
            continue
        mid_x = x + (l - 1) // 2
        supp = (mid_x, y - 1)
        if supp not in grid:
            return False, f"Brick {idx} at ({x},{y},{l}) not supported at {supp}", 0, 0

    # 条件4: すべての穴が覆われているか
    for a, b in info["holes"]:
        if (a, b) not in grid:
            return False, f"Hole ({a},{b}) not covered", 0, 0

    total_cost = sum(info["costs"][l] for x, y, l in bricks)
    score = max(0, W * H * info["costs"][1] - total_cost + 1)
    return True, "OK", total_cost, score

def run_program(exe_path, input_str):
    res = subprocess.run([exe_path], input=input_str, capture_output=True, text=True)
    return res.stdout, res.stderr

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python tester.py <exe1> [exe2] ... [--cases N]")
        sys.exit(1)

    exes = []
    max_cases = None
    sample_dir = os.path.join(os.path.dirname(__file__), "..", "sample_case")
    
    i = 1
    while i < len(sys.argv):
        arg = sys.argv[i]
        if arg == "--cases" and i + 1 < len(sys.argv):
            max_cases = int(sys.argv[i + 1])
            i += 2
        elif arg.startswith("--sample-dir=") and len(arg) > len("--sample-dir="):
            sample_dir = arg.split("=", 1)[1]
            i += 1
        else:
            exes.append(arg)
            i += 1

    case_files = sorted(glob.glob(os.path.join(sample_dir, "*.txt")))
    if not case_files:
        print(f"Error: No sample case files found in {sample_dir}")
        sys.exit(1)

    if max_cases is not None:
        case_files = case_files[:max_cases]

    num_tests = len(case_files)
    print(f"Running {num_tests} test cases from {sample_dir}...")

    total_scores = {exe: 0 for exe in exes}
    total_costs = {exe: 0 for exe in exes}

    for idx, case_file in enumerate(case_files):
        case_name = os.path.basename(case_file)
        inp, info = load_input(case_file)
        print(f"--- [{idx+1}/{num_tests}] {case_name} ---")
        for exe in exes:
            out, err = run_program(exe, inp)
            ok, msg, cost, score = validate_and_score(out, info)
            if not ok:
                print(f"  {exe}: WA ({msg})")
            else:
                total_scores[exe] += score
                total_costs[exe] += cost
                print(f"  {exe}: Cost={cost}, Score={score}")

    print("\n=== Summary ===")
    for exe in exes:
        avg_score = total_scores[exe] / num_tests
        avg_cost = total_costs[exe] / num_tests
        print(f"{exe}: Avg Cost = {avg_cost:.1f}, Avg Score = {avg_score:.1f}")
