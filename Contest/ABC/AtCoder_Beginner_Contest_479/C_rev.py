from collections import defaultdict

N, W = map(int, input().split())

dp = defaultdict(int)
dp[0] = 0

for i in range(N):
    A, B = map(int, input().split())

    prev_dp = dp.copy()

    for w, v in prev_dp.items():
        if w + B <= W:
            dp[w + B] = max(dp[w + B], v + A)

print(max(dp.values()))