N = int(input())
A = list(map(int,input().split()))

# 1つ前の操作1の時の数列の先頭に対象の数字が入れられる
# 愚直階
"""
from collections import deque
import heapq
B1 = deque()
B2 = deque()

for a in A:
    union2 = B2.copy()
    union1 = B1.copy()

    union2.append(a)
    B1 = union2.copy()
    union1.appendleft(a)
    B2 = union1.copy()

print(*B2)
"""

# Nが偶数か奇数によって最終的な並びが変わって、偶数なら大きい順に偶数番目の数字を並べた後、小さい順に奇数番目の数字を並べる
# Nが奇数なら逆

from collections import deque
import heapq
ans = deque()

if N % 2 == 0:
    for i in range(N):
        if i % 2 == 0:
            ans.append(A[i])
        else:
            ans.appendleft(A[i])
else:
    for i in range(N):
        if i % 2 == 1:
            ans.append(A[i])
        else:
            ans.appendleft(A[i])

print(*ans)