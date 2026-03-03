N = int(input())
A = list(map(int,input().split()))

# 見たことある問題、DP(LIS)で解ける
# 1ずつ増加させないといけない

dp = {}
ans = 0

for a in A:
    dp[a] = dp.get(a-1,0) + 1
    # print(dp)

    ans = max(ans,dp[a])

print(ans)
