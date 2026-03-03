import sys
sys.setrecursionlimit(10**7)
from collections import deque
import heapq

N, M = map(int,input().split())

graph = [[] for i in range(N+1)]
for i in range(M):
    A, B = map(int,input().split())
    graph[A].append(B)
    graph[B].append(A)

dist = [-1]*(N+1)
dist[1] = 0
que = deque()
que.append(1)

while len(que) >= 1:
    pos = que.popleft()
    for next in graph[pos]:
        if dist[next] == -1:
            dist[next] = dist[pos] + 1
            que.append(next)

for i in range(1,N+1):
    print(dist[i])