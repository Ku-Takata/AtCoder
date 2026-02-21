N, K = map(int,input().split())
A = list(map(int,input().split()))

import sys
sys.setrecursionlimit(10**7)

def binary_search(list,left,right):
    cnt = 0
    mid = (left + right) // 2
    for i in range(N):
        cnt += mid // A[i]

    if left == right:
        return left

    if cnt < K:
        return binary_search(list,mid+1,right)
    elif cnt >= K:
        return binary_search(list,left,mid)

print(binary_search(A,1,10**9))