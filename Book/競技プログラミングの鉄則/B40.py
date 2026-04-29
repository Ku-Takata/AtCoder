N = int(input())
A = list(map(int,input().split()))

for i in range(N):
    A[i] %= 100

from collections import Counter
import math

cnt_A = Counter(A)
ans = 0

ans += math.comb(cnt_A[0],2)
ans += math.comb(cnt_A[50],2)

for i in range(1,50):
    mod = 100 - i
    ans += cnt_A[i]*cnt_A[mod]

print(ans)