Q = int(input())

from collections import deque
import heapq
que = []

for i in range(Q):
    query = list(map(int,input().split()))

    if query[0] == 1:
        heapq.heappush(que,query[1])
    elif query[0] == 2:
        print(que[0])
    else:
        heapq.heappop(que)