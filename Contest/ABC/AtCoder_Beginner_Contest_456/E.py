T = int(input())
for i in range(T):
    N, M = map(int,input().split())
    UV = [[] for _ in range(N+1)]
    for _ in range(M):
        U, V = map(int,input().split())
        UV[U].append(V)
        UV[V].append(U)
    print(UV)

    W = int(input())
    S = [input() for _ in range(N)]

# つまり、高橋がずっと休日に居られるように都市を移動できるかどうか
# すごいハックやね、私もそうしたいです
# 双方向グラフでoが常にあるところに行けるかどうかを判定したい
# Wは小さいから1つ1つのo確認はそこまで計算量大きくないのかな
# oがあるところを選べるかどうかはDPでできそう

    dp = [[False]*N for _ in range(W)]

    # その都市のその日がoかつ移動可能ならTrueにする、途中でTrueにできなければNoを返す
    for j in range(W):
        if j == 0:
            for first in range(N):
                if S[first][0] == "o":
                    dp[0][first] = True
            continue
        print(j)
        for day in range(W):
            if dp[j-1][day] == True:
                UV[day]
        print(dp)