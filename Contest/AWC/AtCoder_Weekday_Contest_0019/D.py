N, M, T = map(int,input().split())

value = 0
cand = []

for i in range(N):
    A, B, cost = map(int,input().split())

    # 乾燥耐性がT以上は問答無用で足す
    if B >= T:
        value += A
    else:
        cand.append([A,cost])

# print(value)
# print(cand)

# ナップザック問題
C = [ None ] * (len(cand) + 1)
V = [ None ] * (len(cand) + 1)

for i in range(len(cand)):
    V[i+1] = cand[i][0]
    C[i+1] = cand[i][1]

inf = float("-inf")
dp = [[inf]*(M+1) for i in range(len(cand)+1)]
dp[0][0] = 0

for i in range(1,len(cand)+1):
    for j in range(0,M+1):
        if j < C[i]:
            dp[i][j] = dp[i-1][j]
        if j >= C[i]:
            if dp[i-1][j-C[i]] != inf:
                val = dp[i-1][j-C[i]] + V[i]
            else:
                val = inf
            dp[i][j] = max(dp[i-1][j], val)

print(value + max(dp[len(cand)]))