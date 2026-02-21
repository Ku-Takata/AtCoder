N = int(input())
A = sorted(list(map(int,input().split())))
Q = int(input())
X = [int(input()) for i in range(Q)]

import bisect

for i in range(Q):
    print(bisect.bisect_left(A,X[i]))