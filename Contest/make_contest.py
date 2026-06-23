import os

def main():
    print("作成するコンテストの種類を選択してください:")
    print("1: ABC (AtCoder Beginner Contest)  [A - G問題]")
    print("2: AWC (AtCoder Weekday Contest)   [A - E問題]")
    print("3: ADT (AtCoder Daily Training)    [A - I問題]")
    print("4: ARC (AtCoder Regular Contest)   [A - F問題]")
    print("5: AHC (AtCoder Heuristic Contest) [A問題 (C++)]")
    print("6: ABC_Virtual (Virtual Participation) [A - G問題]")
    print("7: Others                          [A - D問題]")
    
    choice = input("番号を入力 (1/2/3/4/5/6/7): ")
    
    # デフォルトの拡張子はPython
    file_extension = ".py"
    
    if choice == '1':
        prefix = "AtCoder_Beginner_Contest"
        parent_dir = "ABC"
        identifier = input("コンテスト番号を入力してください (例: 340): ")
        problems = ["A", "B", "C", "D", "E", "F", "G"]
    elif choice == '2':
        prefix = "AtCoder_Weekday_Contest"
        parent_dir = "AWC"
        identifier = input("コンテスト番号を入力 (例: 0015): ")
        problems = ["A", "B", "C", "D", "E"]
    elif choice == '3':
        prefix = "AtCoder_Daily_Training"
        parent_dir = "ADT"
        identifier = input("日付などを入力 (例: 20260227): ")
        problems = ["A", "B", "C", "D", "E", "F", "G", "H", "I"]
    elif choice == '4':
        prefix = "AtCoder_Regular_Contest"
        parent_dir = "ARC"
        identifier = input("コンテスト番号を入力してください (例: 170): ")
        problems = ["A", "B", "C", "D", "E", "F"]
    elif choice == '5':
        prefix = "AtCoder_Heuristic_Contest"
        parent_dir = "AHC"
        identifier = input("コンテスト番号を入力してください (例: 030): ")
        problems = ["A"]
        file_extension = ".cpp"  # AHCはC++ファイルを作成
    elif choice == '6':
        prefix = "ABC_Virtual"
        parent_dir = "ABC_Virtual"
        identifier = input("コンテスト番号または日付を入力してください (例: 340 / 20260623): ")
        problems = ["A", "B", "C", "D", "E", "F", "G"]
    elif choice == '7':
        prefix = "Others"
        parent_dir = "Others"
        identifier = input("コンテスト名などを入力してください: ")
        problems = ["A", "B", "C", "D"]
    else:
        print("無効な選択です。処理を終了します。")
        return

    # フォルダ名を指定の形式に作成
    dir_name = os.path.join(parent_dir, f"{prefix}_{identifier}")

    try:
        # フォルダの作成
        os.makedirs(dir_name, exist_ok=True)
        print(f"\nDirectory created: {dir_name}")

        # 問題ファイルの作成
        for p in problems:
            # 選択された拡張子（.py または .cpp）を使用
            file_path = os.path.join(dir_name, f"{p}{file_extension}")

            # 既存のファイルを上書きしないように確認
            if not os.path.exists(file_path):
                with open(file_path, "w", encoding="utf-8") as f:
                    pass # 中身は空
                print(f"  - Created: {p}{file_extension}")
            else:
                print(f"  - Already exists: {p}{file_extension}")

        print("\nAll tasks completed successfully!")

    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()