N, Q = map(int,input().split())

# OSのバージョン以下の数をカウントしたリストを管理する

from collections import deque
import heapq
OS = [[i,1] for i in range(1,N+1)]
heapq.heapify(OS)

for i in range(Q):
    X,Y = map(int,input().split())
    cnt = 0

    min_ver = heapq.heappop(OS)
    while min_ver[0] <= X:
        if min_ver[0] <= X:
            cnt += 1
        else:
            OS.heappush(OS,[Y,cnt])
    print(OS)
    print(cnt)