N = int(input())
A = {}

for i in range(N):
    a = int(input())

    if a not in A:
        A[a] = 1
    else:
        A[a] += 1

import math
ans = 0

for v in A.values():
    ans += math.comb(v,2)

print(ans)