N, M = map(int,input().split())

import sys
sys.setrecursionlimit(10**7)

def dfs(root):
    global cnt
    if visited[root]:
        return
    visited[root] = True
    cnt += 1

    for next_node in graph[root]:
        dfs(next_node)

graph = [[] for i in range(N+1)]

for i in range(M):
    A, B = map(int,input().split())
    graph[A].append(B)

cnt = 0

for i in range(1,N+1):
    visited = [False for _ in range(N+1)]
    dfs(i)

print(cnt)