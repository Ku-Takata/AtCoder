N, M = map(int,input().split())

import sys
sys.setrecursionlimit(10**7)

def dfs(pos, graph, visited):
    visited[pos] = True
    if color[pos] == "b":
        return True
    for i in graph[pos]:
        if not visited[i]:
            if dfs(i, graph, visited):
                return True
    return False

graph = [[] for i in range(N)]
for i in range(M):
    X,Y = map(int,input().split())
    graph[X-1].append(Y-1)

Q = int(input())
color = ["w" for i in range(N)]
for i in range(Q):
    num,v = map(int,input().split())
    if num == 1:
        color[v-1] = "b"
    else:
        visited = [False]*N
        if dfs(v,graph,visited) is True:
            print("Yes")
        else:
            print("No")
