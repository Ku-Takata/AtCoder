from collections import defaultdict
from itertools import combinations_with_replacement
import math

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

# とりあえず一番上にある穴の下をc1で埋める
# 次は隣がつながっている箇所はその幅に合わせたブロックの組み合わせにする

ab.sort(key=lambda x: x[1], reverse=True)
# ans = one_block([])

# 穴のx座標をkeyとして、y座標をvalueとするデータ構造
data = defaultdict(list)
for a,b in ab:
    data[a].append(b)
data_sorted = sorted(data.items(), key=lambda x:x[0], reverse=True)
# print(data_sorted)
M = 0
ans = []
neighbor = 1
width = [1,3,5,7,9]

for i in range(len(data_sorted)-1):
    k,v = data_sorted[i][0], data_sorted[i][1][0]
    if data_sorted[i][0] == data_sorted[i+1][0]+1:
        neighbor += 1
    elif neighbor > 1 or neighbor == 9:
        if neighbor == 3:
            comb = [(3,)]
        elif neighbor == 5:
            comb = [(5,)]
        elif neighbor == 7:
            comb = [(7,)]
        elif neighbor == 9:
            comb = [(9,)]
        else:
            all_comb = list(combinations_with_replacement(width,2))
            comb = [c for c in all_comb if sum(c) == neighbor]
        neighbor = 1
        # print(comb)

        if comb == [(1,1)]:
            for i in range(v+1):
                ans.append([k,v-i,1])
            for i in range(v+1):
                ans.append([k+1,v-i,1])
        else:
            for i in range(v+1):
                ans.append([k,v-i,comb[0][0]])
            if len(comb[0]) > 1:
                for i in range(v+1):
                    ans.append([k,v-i,comb[0][1]])

    else:
        for i in range(v+1):
            ans.append([k,v-i,1])

print(len(ans))
for a,b,c in ans:
    print(a,b,c)

# print(data_sorted)




# https://github.com/Ku-Takata/AtCoder/blob/0bbb4d83c8ef0bd886ed7e38ce1fce37ef5c0230/atcoder_py_snipets.json