N = int(input())
H = list(map(int, input().split()))

dp = [0] * N

dp[1] = abs(H[0]-H[1])

for i in range(2,N):
    cost1 = dp[i-1] + abs(H[i-1]-H[i])
    cost2 = dp[i-2] + abs(H[i-2]-H[i])

    dp[i] = min(cost1,cost2)

print(dp[-1])