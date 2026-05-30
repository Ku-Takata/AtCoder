N, M = map(int,input().split())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

# 貪欲にシャリの重さに対して、なるべく限界値まで思いネタを乗せたい。

A.sort()
B.sort()

s,n = 0,0
cnt = 0

while s < N and n < M:
    if A[s]*2 >= B[n]:
        cnt += 1
        s += 1
        n += 1
    else:
        s += 1

print(cnt)