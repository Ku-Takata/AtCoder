import subprocess
import os
import glob
import re
import time

# ── 設定 ──────────────────────────────────────────────────
INPUT_DIR = "./O25rQjiK/tools/in"      # 予め生成されている入力があるフォルダ
OUTPUT_DIR_A = "./output_A"            # Code A の出力結果を保存するフォルダ
OUTPUT_DIR_B = "./output_B"            # Code B の出力結果を保存するフォルダ

# ⚠️ 比較したいファイル名に合わせて、ここを書き換えてください
CPP_SOURCE_A = "temp.cpp"
EXE_NAME_A = "temp.exe"

CPP_SOURCE_B = "A_submit.cpp"
EXE_NAME_B = "A_submit.exe"

REPORT_FILE = "comparison_result.txt"  # 結果をまとめるテキストファイル

# 💡 追加：実行するケース数を指定（None にすると全ケースを実行、10 にすると最初の10ケースのみ実行）
NUM_CASES = 1000
# ──────────────────────────────────────────────────────────

def compile_code(source, exe):
    print(f"Compiling {source}...")
    compile_cmd = ["g++", "-O3", "-std=c++17", "-DLOCAL_DEBUG", source, "-o", exe]
    if subprocess.run(compile_cmd).returncode != 0:
        print(f"Error: Compilation failed for {source}.")
        return False
    return True

def run_and_get_score(exe_name, in_path, out_path):
    start_p = time.perf_counter()
    with open(in_path, "r") as inf, open(out_path, "w") as outf:
        res_cpp = subprocess.run(
            [f"./{exe_name}"], 
            stdin=inf, 
            stdout=outf, 
            stderr=subprocess.PIPE, 
            text=True, 
            errors='replace'
        )
    exec_time = time.perf_counter() - start_p
    debug_out = res_cpp.stderr.strip()
    
    score = None
    state = "UNKNOWN"

    # 1. 複数パラメータ探索（Best Solution Selected!）の判定
    match_best = re.search(r"Best Solution Selected!\s*cmd_len:\s*(\d+)", debug_out)
    if match_best:
        parsed_len = int(match_best.group(1))
        is_success = False
        is_fallback = False
        for line in debug_out.split('\n'):
            if f"cmd_len: {parsed_len}" in line and "Best Solution Selected!" not in line:
                if "Goal Reached!" in line:
                    is_success = True
                elif "Fallback Greedy" in line:
                    is_fallback = True

        if is_success:
            score = parsed_len
            state = "SUCCESS"
        elif is_fallback:
            score = parsed_len
            state = "FALLBACK"
        else:
            score = 2000
            state = "INCOMPLETE"

    # 2. 単一実行（新・旧フォーマット両対応）の判定
    else:
        match_cpp = re.search(r"cmd_len:\s*(\d+)", debug_out)
        parsed_len = int(match_cpp.group(1)) if match_cpp else None

        if "Goal Reached!" in debug_out:
            score = parsed_len
            state = "SUCCESS"
        elif "Fallback Greedy" in debug_out:
            score = parsed_len
            state = "FALLBACK"
        elif "Partial Output" in debug_out or "Failed" in debug_out or "Timeout" in debug_out:
            score = 2000
            state = "INCOMPLETE"
        else:
            if parsed_len is not None:
                score = parsed_len
                state = "SUCCESS"

    return score, exec_time, state

def get_status_label(state):
    if state == "SUCCESS": return "完走"
    if state == "FALLBACK": return "代替"
    if state == "INCOMPLETE": return "途中"
    return "不明"

def main():
    os.makedirs(OUTPUT_DIR_A, exist_ok=True)
    os.makedirs(OUTPUT_DIR_B, exist_ok=True)

    # コンパイル
    if not compile_code(CPP_SOURCE_A, EXE_NAME_A): return
    if not compile_code(CPP_SOURCE_B, EXE_NAME_B): return

    input_files = sorted(glob.glob(f"{INPUT_DIR}/*.txt"))
    if not input_files:
        print(f"Error: No input files found in {INPUT_DIR}.")
        return

    # 💡 追加：指定されたケース数にスライスする処理
    if NUM_CASES is not None:
        input_files = input_files[:NUM_CASES]

    print(f"\nRunning tests and comparing on {len(input_files)} cases...\n")

    # 集計用変数
    valid_cases = 0
    total_score_A = 0
    total_score_B = 0
    
    total_time_A = 0.0
    total_time_B = 0.0
    max_time_A = 0.0
    max_time_B = 0.0
    
    # 状態カウント
    completed_A = 0; fallback_A = 0
    completed_B = 0; fallback_B = 0
    incomplete_cases_A = []
    incomplete_cases_B = []
    
    wins_A = 0
    wins_B = 0
    draws = 0

    # ログファイルを開く
    with open(REPORT_FILE, "w", encoding="utf-8") as report:
        report.write("=== AHC 066 Comparison Report ===\n")
        report.write(f"Code A: {CPP_SOURCE_A}\n")
        report.write(f"Code B: {CPP_SOURCE_B}\n")
        report.write("-" * 60 + "\n")
        report.write("【ケース別結果】\n")

        for in_path in input_files:
            filename = os.path.basename(in_path)
            
            # Code A の実行
            out_path_A = f"{OUTPUT_DIR_A}/{filename}"
            score_A, time_A, state_A = run_and_get_score(EXE_NAME_A, in_path, out_path_A)
            total_time_A += time_A
            if time_A > max_time_A: max_time_A = time_A
            
            if state_A == "SUCCESS": completed_A += 1
            elif state_A == "FALLBACK": completed_A += 1; fallback_A += 1
            else: incomplete_cases_A.append(filename)

            # Code B の実行
            out_path_B = f"{OUTPUT_DIR_B}/{filename}"
            score_B, time_B, state_B = run_and_get_score(EXE_NAME_B, in_path, out_path_B)
            total_time_B += time_B
            if time_B > max_time_B: max_time_B = time_B
            
            if state_B == "SUCCESS": completed_B += 1
            elif state_B == "FALLBACK": completed_B += 1; fallback_B += 1
            else: incomplete_cases_B.append(filename)

            # 比較と結果の文字列作成
            if score_A is not None and score_B is not None:
                valid_cases += 1
                total_score_A += score_A
                total_score_B += score_B

                if score_A < score_B:
                    winner = "A 勝利"
                    diff = score_B - score_A
                    wins_A += 1
                elif score_B < score_A:
                    winner = "B 勝利"
                    diff = score_A - score_B
                    wins_B += 1
                else:
                    winner = "引き分け"
                    diff = 0
                    draws += 1

                # コンソールとファイルへの出力フォーマット
                status_A_str = get_status_label(state_A)
                status_B_str = get_status_label(state_B)
                
                result_str = (f"Case {filename} | "
                              f"A: {score_A:>6,} [{status_A_str}] ({time_A:.2f}s) | "
                              f"B: {score_B:>6,} [{status_B_str}] ({time_B:.2f}s) | "
                              f"{winner} (差: {diff:,})")
                print(result_str)
                report.write(result_str + "\n")
            else:
                err_str = f"Case {filename} | エラー: スコア取得失敗"
                print(err_str)
                report.write(err_str + "\n")

        # 最終サマリーの計算と作成
        if valid_cases > 0:
            avg_time_A = total_time_A / valid_cases
            avg_time_B = total_time_B / valid_cases
            
            summary = (
                "\n" + "=" * 50 + "\n"
                "【最終サマリー】\n"
                f"有効ケース数 : {valid_cases}\n"
                f"勝敗         : Code A {wins_A}勝 / Code B {wins_B}勝 / 引き分け {draws}\n"
                f"合計スコア   : A = {total_score_A:,} / B = {total_score_B:,}\n"
                f"平均スコア   : A = {total_score_A / valid_cases:,.2f} / B = {total_score_B / valid_cases:,.2f}\n"
                "--------------------------------------------------\n"
                f"成功完走数   : A = {completed_A - fallback_A}/{valid_cases} / B = {completed_B - fallback_B}/{valid_cases}\n"
                f"代替完走数   : A = {fallback_A}/{valid_cases} / B = {fallback_B}/{valid_cases}\n"
                f"平均実行時間 : A = {avg_time_A:.3f}s / B = {avg_time_B:.3f}s\n"
                f"最大実行時間 : A = {max_time_A:.3f}s / B = {max_time_B:.3f}s\n"
            )

            if incomplete_cases_A or incomplete_cases_B:
                summary += "--------------------------------------------------\n"
                if incomplete_cases_A:
                    summary += f"[Code A 完走失敗(スコア2000適用)ケース]\n{', '.join(incomplete_cases_A)}\n"
                if incomplete_cases_B:
                    summary += f"[Code B 完走失敗(スコア2000適用)ケース]\n{', '.join(incomplete_cases_B)}\n"
            
            summary += "=" * 50 + "\n"
            
            print(summary)
            report.write(summary)
            print(f"\n※ 詳細な結果は '{REPORT_FILE}' に保存されました。")

if __name__ == "__main__":
    main()