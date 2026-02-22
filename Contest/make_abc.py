import os
import sys

def main():
    # 引数（コンテスト番号）がない場合の処理
    if len(sys.argv) < 2:
        print("Usage: python make_abc.py <contest_number>")
        print("Example: python make_abc.py 445")
        return

    contest_no = sys.argv[1]
    # フォルダ名を指定の形式に作成
    dir_name = f"AtCoder_Beginner_Contest_{contest_no}"

    try:
        # フォルダの作成
        os.makedirs(dir_name, exist_ok=True)
        print(f"Directory created: {dir_name}")

        # A.py から E.py まで空ファイルを作成
        problems = ["A", "B", "C", "D", "E"]
        for p in problems:
            file_path = os.path.join(dir_name, f"{p}.py")

            # 既存のファイルを上書きしないように確認
            if not os.path.exists(file_path):
                with open(file_path, "w", encoding="utf-8") as f:
                    pass # 中身は空
                print(f"  - Created: {p}.py")
            else:
                print(f"  - Already exists: {p}.py")

        print("\nAll tasks completed successfully!")

    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()