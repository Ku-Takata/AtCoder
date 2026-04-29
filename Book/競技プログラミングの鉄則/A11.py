N, X = map(int,input().split())
A = list(map(int,input().split()))

import bisect

print(bisect.bisect_left(A,X)+1)