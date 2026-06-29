N, M = map(int,input().split())
A = [list(map(int,input().split())) for i in range(N)]

# 全探索しても100*100で10**4で済む

import itertools
select = [i for i in range(M)]

ans = 0

for s in itertools.combinations(select,2):
    s1,s2 = s
    total = 0
    for i in range(N):
        total += max(A[i][s1],A[i][s2])

    ans = max(ans,total)

print(ans)