N = int(input())
A = list(map(int,input().split()))

from collections import Counter
import math

cnt_A = Counter(A)
ans = 0

for k,v in cnt_A.items():
    if v >= 3:
        ans += math.comb(v,3)

print(ans)