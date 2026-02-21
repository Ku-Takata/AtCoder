N, S = map(int,input().split())
A = list(map(int,input().split()))

dp = [[False]*(S+1) for i in range(N+1)]

for i in range(N+1):
    if i == 0:
        dp[0][0] = True
        continue
    for j in range(S+1):
        if dp[i-1][j] == True:
            dp[i][j] = True
            if j+A[i-1] <= S:
                dp[i][j+A[i-1]] = True

if dp[-1][-1]:
    print("Yes")
else:
    print("No")