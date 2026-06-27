T = int(input())

# つまりX_iのペナルティを受けてもY_iの嬉しさの方が大きかったら変更するのが良い
# でもこれどうやって全体を考慮して変更したらいいんだ？
# dpっぽさはあるよね
# その日が雨か晴れで嬉しさがどうなるかを見て、最終的に最大化するよう選べばよい

for i in range(T):
    N = int(input())
    S = input()
    X = list(map(int,input().split()))
    Y = list(map(int,input().split()))

    # i日目の、1つ目が晴の時の最大値、2つ目が雨の時の最大値
    dp = [[0,0] for i in range(N + 1)]

    cost_s,cost_r = 0,0
    if S[0] == "R":
        cost_r = 0
    else:
        cost_r = X[0]
    if S[0] == "S":
        cost_s = 0
    else:
        cost_s = X[0]

    dp[1][0] = -cost_s
    dp[1][1] = -cost_r

    for i in range(2, N + 1):
        if S[i-1] == "S":
            cost_s = 0
        else:
            cost_s = X[i-1]
        if S[i-1] == "R":
            cost_r = 0
        else:
            cost_r = X[i-1]

        # i日目を晴れにする場合、i-1日目が雨ならY_i-2を足す
        # i日目を雨にする場合はYから加算は無い
        dp[i][0] = max(dp[i-1][0], dp[i-1][1] + Y[i-2]) - cost_s
        dp[i][1] = max(dp[i-1][0], dp[i-1][1]) - cost_r

    print(max(dp[N][0], dp[N][1]))