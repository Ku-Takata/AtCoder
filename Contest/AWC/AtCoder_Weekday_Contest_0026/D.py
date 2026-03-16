N, K = map(int,input().split())

# LR別々でソートして、数が小さい順からリストで入れる
# このとき、10**9個の要素を持つリストで管理するのではなく、小さい順の数を入れる

import sys
sys.setrecursionlimit(10**7)

L = []
R = []

for i in range(N):
    l, r = map(int,input().split())
    L.append(l)
    R.append(r)

L = sorted(L)
R = sorted(R)
dp = []
i,j = 0,0

while i < N or j < N:
    if i == N:
        dp.append([R[j],-1])
        j += 1
    elif j == N:
        dp.append([L[i],1])
        i += 1
    elif L[i] <= R[j]:
        dp.append([L[i],1])
        i += 1
    elif L[i] > R[j]:
        dp.append([R[j],-1])
        j += 1

# print(dp)
total = 0
ans = 0

for i in range(N*2):
    if i > 0:
        current_pos = dp[i][0]
        prev_pos = dp[i-1][0]
        if total >= K:
            ans += (current_pos - prev_pos)

    total += dp[i][1]

print(ans)