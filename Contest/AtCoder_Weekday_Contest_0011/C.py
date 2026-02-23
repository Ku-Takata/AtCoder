N, K = map(int,input().split())
A = list(map(int,input().split()))

cnt = 0
total = 0

for i in range(N):
    if (A[i] | K) == K:
        cnt += 1
        total |= A[i]

if total == K and cnt > 0:
    print(cnt)
else:
    print(-1)


# 問題文の意味が分からん。
# まずAの要素は10進数だけど、実際には2進数ということ？
# それでOR演算をして合計がKになるような最大部分列長を求めるということ？