N, K = map(int,input().split())
H = list(map(int,input().split()))
inf = float("inf")
dp = [inf]*N
dp[0] = 0

for i in range(1,N):
    for j in range(1,K+1):
        if i - j < 0:
            break
        dp[i] = min(dp[i], dp[i-j] + abs(H[i]-H[i-j]))

# print(dp)
print(dp[-1])