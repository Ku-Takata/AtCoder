N, Q = map(int,input().split())

import sys
sys.setrecursionlimit(10**7)

def dfs(root):
    visited[root] = True

    for next_node in graph[root]:
        if visited[next_node]:
            continue

        counter[next_node] += counter[root]
        dfs(next_node)

graph = [[] for i in range(N+1)]

for i in range(N-1):
    a,b = map(int,input().split())
    graph[a].append(b)
    graph[b].append(a)

counter = [0 for i in range(N+1)]
visited = [False for i in range(N+1)]

for i in range(Q):
    p,x = map(int,input().split())
    counter[p] += x

dfs(1)
print(*counter[1:])