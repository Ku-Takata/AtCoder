# x 番の組織が親組織に提出する報告書の枚数を返す関数
def num_report(children, x):
    # ここに追記して再帰関数を実装する
    # base case
    if len(children) == 0:
        return 1

    watasu = 1
    for i in children[x]:
        watasu += num_report(children, i)
    return watasu

# これ以降の行は変更しなくてよい
N = int(input())
# p[i] : 組織 i の親組織
# 組織 0 の親組織は存在しないので -1 を入れておく
# 組織 0 に相当する部分は入力で与えられないため、リスト [-1] を作成して "+" 演算子で結合する
p = [-1] + [int(c) for c in input().split()]

# children[i] = 組織 i の子組織のリスト
children = [[] for _ in range(N)]
for i in range(1, N):
    parent = p[i] # 組織 i の親組織は parent
    children[parent].append(i) # parent の子組織リストに i を追加

# 各組織について答えを計算し出力する
for i in range(N):
    res = num_report(children, i)
    print(res)
