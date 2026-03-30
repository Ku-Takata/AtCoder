N, M = map(int,input().split())

graph = [[] for i in range(N+1)]

for i in range(M):
    u,v = map(int,input().split())
    graph[u].append(v)
    graph[v].append(u)

# print(graph)
visited = [False for i in range(N+1)]
cnt = 0

def dfs(pos):
    if visited[pos]:
        return
    else:
        visited[pos] = True

    for i in graph[pos]:
        dfs(i)

for i in range(1,N+1):
    if not visited[i]:
        dfs(i)
        cnt += 1

print(cnt)