W, H, K = map(int,input().split())
c1, c3, c5, c7, c9 = map(int,input().split())
ab = [list(map(int,input().split())) for i in range(K)]

# レンガは支えが必要で、レンガの中心のy座標の下にレンガが必要
# できる限りコストを低く抑えたい
# 1マス当たりのコストは計算する必要がありそう

# 基本的にc1が1番安いので、一旦c1で穴を埋める
# その後に支えが必要な箇所はまずは幅を大きいレンガを選んでみる
# それで隣の穴1つをカバーできない範囲にあるなら、幅を大きくするのではなく、別でレンガを用意することを考える
# これを繰り返していくことでそこそこの答えになりそう
# これ最後に出力された奴だけ評価するから、printして変化をビジュアライザーで確認できるってことか
# 一旦コスト無視して大きさだけで決めるか
# いや全ての穴とその下をc1レンガで埋めるか
# 上から下に処理した方が良さそう

# まず条件を満たしているかどうかを評価する関数が必要
# 次に穴を1で埋める関数
# 次に調整をする関数

# 条件確認関数
def ok(M,ans):
    None

# 穴をc1で埋める関数
def one_block(ans):
    for i in range(K):
        a,b = ab[i][0],ab[i][1]
        ans.append([a,b,1])
    return ans

# 改善関数
def adjust(ans):
    None

ab.sort(key=lambda x: x[1], reverse=True)
ans = one_block([])

ab_copy = ab.copy()
ab_x = [ab[i][0] for i in range(K)]
ab_x = set(ab_x)
# print(ab_x)

print(len(ab_x)*40)
for a in ab_x:
    for b in range(40):
        print(a,b,1)



"""
# 出力
print(len(ans))
for i in range(len(ans)):
    print(*ans[i])

"""


# https://github.com/Ku-Takata/AtCoder/blob/0bbb4d83c8ef0bd886ed7e38ce1fce37ef5c0230/atcoder_py_snipets.json