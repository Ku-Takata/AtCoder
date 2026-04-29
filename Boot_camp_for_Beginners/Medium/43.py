X = int(input())

# 基本的にはジャンプして、もしジャンプした結果が次のジャンプ幅よりも近くなってしまう場合は立ち止まりが必要と仮定する
# これ以外考えれなくてギブした。
# どうやらi秒後には、毎回ジャンプした場合の座標までを1刻みで全て移動することが可能であると証明できる
# なので、毎回ジャンプした先にX以上の座標に行けたとき、最小の時間があると言える

x = 0
ans = 0
"""
for i in range(1,X+1):
    ans += 1
    if x+i == X:
        break
    elif X-(x+i) < i+1:
        continue
    else:
        x += i

print(ans)
"""

for i in range(1,X+1):
    ans += 1
    x += i
    if x >= X:
        print(ans)
        exit()