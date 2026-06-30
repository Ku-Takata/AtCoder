import sys
sys.setrecursionlimit(10**7)
N, M = map(int, input().split())

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

visited = [0 for i in range(N)]

def dfs(cur):
    visited[cur] = 1

    for next in graph[cur]:
        if not visited[next]:
            dfs(next)

ans = "Yes"

if N != M:
    ans = "No"

for i in range(N):
    if len(graph[i]) != 2:
        ans = "No"
        break

dfs(0)
if sum(visited) != N:
    ans = "No"

print(ans)