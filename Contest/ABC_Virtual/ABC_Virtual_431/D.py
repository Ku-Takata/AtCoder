N = int(input())
WHB = [list(map(int,input().split())) for i in range(N)]

# 部品をHかBのどちらに取り付けるかを選択して、倒れないように嬉しさを最大化したい
# Nが500以下なので、ループ回数自体はたくさんできそう
# バックパックみたいな問題じゃない？DPっぽさあるけど分からん
# 一旦全部Bにして、その後にDPでどれを選択したときにその重さ時点で最適かを選んでいけば良さそう
# 基本的にBにしておけば倒れないから、Hにするときは嬉しさがBを上回っているときだけ選べばよい

H_weight,B_weight = 0,sum(WHB[i][0] for i in range(N))
# 頭の重さでDP
dp = [0 for i in range(B_weight//2 + 1)]
dp[0] = sum(WHB[i][2] for i in range(N))
# 頭の重さの総和
total_W = 0

for i in range(N):
    W,H,B = WHB[i][0],WHB[i][1],WHB[i][2]

    total_W += W
    # DPの値があるところに対してインデックスをW分ずらして、元の値より大きかったら更新するというのをしたい
    for j in range(B_weight//2 - W,-1,-1):
        if dp[j] != 0:
            dp[j+W] = max(dp[j]+H-B,dp[j+W])

# print(dp)
print(max(dp))