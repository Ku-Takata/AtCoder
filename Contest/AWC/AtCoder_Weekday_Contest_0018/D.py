N, K = map(int,input().split())
C = list(map(int,input().split()))

dp = [False] * (K+1)
dp[0] = True

for price in C:
    for i in range(K, price-1, -1):
        if dp[i-price]:
            dp[i] = True

ans = 0
for i in range(K+1):
    if dp[i]:
        ans = i

print(ans)