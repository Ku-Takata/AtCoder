import sys
sys.setrecursionlimit(10**7)
N, M = map(int, input().split())

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)

# 連結成分の頂点数の最大を求める

visited = [False for i in range(N)]
group = []

def dfs(cur):
    visited[cur] = True
    group.append(cur)

    for next in graph[cur]:
        if not visited[next]:
            dfs(next)

    return group

dfs(0)
print(len(group))