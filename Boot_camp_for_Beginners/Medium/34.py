N, K = map(int,input().split())
A = list(map(int,input().split()))

# 同じ値の要素は無いから簡単
from math import ceil

ans = 1 + ceil((N-K)/(K-1))
print(ans)