N, X = map(int,input().split())
A = list(map(int,input().split()))

# 全ての要素間の最大公約数を求めたらいけそう
from math import gcd

A.append(X)
A.sort()
dif = []

for i in range(N):
    dif.append(A[i+1]-A[i])

ans = gcd(*dif)
print(ans)