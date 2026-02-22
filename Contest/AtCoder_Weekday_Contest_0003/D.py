N, K, M = map(int,input().split())
A = list(map(int,input().split()))

cs = [0]
total = 0

for i in range(N):
    total += A[i]
    cs.append(total)

cnt = 0
# print(cs)

import bisect

for i in range(K,N+1):
    achieve = cs[i] - M
    cnt_achieve = bisect.bisect_right(cs, achieve, 0, i-K+1)
    cnt += cnt_achieve

print(cnt)