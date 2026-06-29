import sys
sys.setrecursionlimit(10**7)

N, M = map(int, input().split())


graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

visited = [False for i in range(N)]

def dfs(cur):
    visited[cur] = True
    for next in graph[cur]:
        if not visited[next]:
            dfs(next)

    return True

ans = 0

for i in range(N):
    if not visited[i]:
        dfs(i)
        ans += 1

print(ans)