N, Q = map(int,input().split())

# OSのバージョン以下の数をカウントしたリストを管理する

from collections import Counter
import heapq
OS = [i for i in range(1,N+1)]
cnt_OS = Counter(OS)
heapq.heapify(OS)

for i in range(Q):
    X,Y = map(int,input().split())
    cnt = 0

    while True:
        min_ver = heapq.heappop(OS)
        if min_ver <= X:
            cnt += cnt_OS[min_ver]
        else:
            heapq.heappush(OS,min_ver)
            break
    cnt_OS[Y] += cnt

    # print(OS)
    # print(cnt_OS)
    print(cnt)