N, D = map(int,input().split())
XY = [list(map(int,input().split())) for i in range(N)]

import sys
sys.setrecursionlimit(10**7)

def dfs(root):
    if visited[root]:
        return
    visited[root] = True

    for next_node in graph[root]:
        dfs(next_node)

graph = [[] for i in range(N)]
visited = [False]*N

for i in range(N-1):
    for j in range(i+1,N):
        dist = ((XY[i][0] - XY[j][0])**2 + (XY[i][1] - XY[j][1])**2)**0.5
        if dist <= D:
            graph[i].append(j)
            graph[j].append(i)

# print(graph)

dfs(0)

for i in range(N):
    if visited[i]:
        print("Yes")
    else:
        print("No")