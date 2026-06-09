import subprocess
import os
import glob
import re
import time

# ── 設定 ──────────────────────────────────────────────────
INPUT_DIR = "./O25rQjiK/tools/in"          # 予め生成されている入力があるフォルダ
OUTPUT_DIR = "./output"        # 自分のコードの出力結果を保存するフォルダ
CPP_SOURCE = "temp.cpp"     # あなたのC++ソースファイル名
EXE_NAME = "A.exe"       # コンパイル後の実行ファイル名
# ──────────────────────────────────────────────────────────

def main():
    # 1. 出力用フォルダの作成
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # 2. C++コードのコンパイル
    print("Compiling C++ code...")
    compile_cmd = ["g++", "-O3", "-std=c++17", "-DLOCAL_DEBUG", CPP_SOURCE, "-o", EXE_NAME]
    if subprocess.run(compile_cmd).returncode != 0:
        print("Error: Compilation failed.")
        return

    # 3. テストケース（inフォルダ内のtxtファイル）の一覧を取得
    input_files = sorted(glob.glob(f"{INPUT_DIR}/*.txt"))
    if not input_files:
        print(f"Error: No input files found in {INPUT_DIR}.")
        return

    total_score = 0
    valid_cases = 0
    max_exec_time = 0.0
    print(f"Running tests on {len(input_files)} cases...\n")

    # 4. 各ケースをループ処理
    for in_path in input_files:
        filename = os.path.basename(in_path)
        out_path = f"{OUTPUT_DIR}/{filename}"

        start_p = time.perf_counter()

        # C++の標準エラー出力(cerr)をキャプチャ
        with open(in_path, "r") as inf, open(out_path, "w") as outf:
            res_cpp = subprocess.run(
                [f"./{EXE_NAME}"], 
                stdin=inf, 
                stdout=outf, 
                stderr=subprocess.PIPE, 
                text=True, 
                errors='replace'
            )

        end_p = time.perf_counter()
        exec_time = end_p - start_p

        if exec_time > max_exec_time:
            max_exec_time = exec_time

        # C++側が画面に出していたデバッグ文を表示
        cpp_debug_output = res_cpp.stderr.strip()
        if cpp_debug_output:
            print(cpp_debug_output)

        # 💡 修正：「len: 数値」の部分から真のスコア（操作文字数）を抽出するように変更
        score = None
        match_cpp = re.search(r"len:\s*(\d+)", cpp_debug_output)
        
        if match_cpp:
            score = int(match_cpp.group(1))
        else:
            # 万が一C++側から取れなかった場合の予備
            vis_cmd = ["cargo", "run", "-r", "--bin", "vis", in_path, out_path]
            res_vis = subprocess.run(vis_cmd, capture_output=True, text=True)
            match_vis = re.search(r"Score\s*=\s*(\d+)", res_vis.stdout)
            if match_vis:
                score = int(match_vis.group(1))

        # スコアの集計と表示
        if score is not None:
            total_score += score
            valid_cases += 1
            print(f"→ Case {filename}: Score = {score:,} ({exec_time:.3f}s)\n")
        else:
            print(f"→ Case {filename}: Could not retrieve score. (Check your program's output)\n")

    # 5. 最終結果の表示
    if valid_cases > 0:
        print("\n" + "═" * 40)
        print(f"Tested Cases : {valid_cases}")
        print(f"Total Score  : {total_score:,}")
        print(f"Average Score: {total_score / valid_cases:,.2f}")
        print(f"Max Exec Time: {max_exec_time:.3f} s")
        print("═" * 40)

if __name__ == "__main__":
    main()