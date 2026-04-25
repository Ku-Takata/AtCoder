N, K = map(int,input().split())
A = list(map(int,input().split()))

# 普通に辞書にして大きいやつをK個選んでやれば良さそう

from collections import Counter

cnt_A = Counter(A)
A_rev = []
K_minus = 0


for k,v in cnt_A.items():
    A_rev.append(k*v)

A_rev.sort(reverse=True)
# print(A_rev)
print(sum(A)-sum(A_rev[:K]))