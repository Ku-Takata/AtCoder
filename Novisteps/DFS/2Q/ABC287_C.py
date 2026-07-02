import sys
sys.setrecursionlimit(10**7)
N, M = map(int, input().split())

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

# パスグラフとは全ての頂点が一直線に繋がれたグラフ

visited = [0 for i in range(N)]

def dfs(cur,state,pre):
    visited[cur] = 1
    if len(graph[cur]) > 2:
        state = False

    for next in graph[cur]:
        if next == pre:
            continue
        if not visited[next]:
            if dfs(next,state,cur):
                state = True
            else:
                state = False
        else:
            state = False

    return state

if dfs(0,True,-1):
    if sum(visited) == N:
        print("Yes")
    else:
        print("No")
else:
    print("No")