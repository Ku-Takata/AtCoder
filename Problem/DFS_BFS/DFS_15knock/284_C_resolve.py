N, M = map(int,input().split())

def dfs(node):
    if visited[node]:
        return

    visited[node] = True

    for next_node in graph[node]:
        dfs(next_node)

graph = [[] for i in range(N+1)]

for i in range(M):
    u,v = map(int,input().split())
    graph[u].append(v)
    graph[v].append(u)

visited = [False for i in range(N+1)]
ans = 0

for i in range(1,N+1):
    if visited[i]:
        continue
    dfs(i)
    ans += 1

print(ans)