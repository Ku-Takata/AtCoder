N, W = map(int,input().split())
AB = [list(map(int,input().split())) for i in range(N)]

dp = [[0]*(W+1) for i in range(N+1)]

for i in range(N):
    for j in range(W+1):
        if j < AB[i][1]:
            dp[i+1][j] = dp[i][j]
        else:
            dp[i+1][j] = max(dp[i][j], dp[i][j-AB[i][1]]+AB[i][0])

print(dp[N][W])