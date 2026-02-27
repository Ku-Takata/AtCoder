import os

def main():
    print("作成するコンテストの種類を選択してください:")
    print("1: ABC (AtCoder Beginner Contest)  [A - G問題]")
    print("2: AWC (AtCoder Weekday Contest)   [A - E問題]")
    print("3: ADT (AtCoder Daily Training)    [A - I問題]")
    
    choice = input("番号を入力 (1/2/3): ")
    
    if choice == '1':
        prefix = "AtCoder_Beginner_Contest"
        identifier = input("コンテスト番号を入力してください (例: 340): ")
        problems = ["A", "B", "C", "D", "E", "F", "G"]
    elif choice == '2':
        prefix = "AtCoder_Weekday_Contest"
        identifier = input("コンテスト番号を入力してください (例: 0015): ")
        problems = ["A", "B", "C", "D", "E"]
    elif choice == '3':
        prefix = "AtCoder_Daily_Training"
        identifier = input("日付などを入力してください (例: 20260227_2): ")
        problems = ["A", "B", "C", "D", "E", "F", "G", "H", "I"]
    else:
        print("無効な選択です。処理を終了します。")
        return

    # フォルダ名を指定の形式に作成
    dir_name = f"{prefix}_{identifier}"

    try:
        # フォルダの作成
        os.makedirs(dir_name, exist_ok=True)
        print(f"\nDirectory created: {dir_name}")

        # 問題ファイルの作成
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