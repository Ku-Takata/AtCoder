import sys
sys.setrecursionlimit(10**7)

N, M = map(int, input().split())

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

for i in range(N):
    graph[i] = list(set(graph[i]))

visited = [False for i in range(N)]

def dfs(cur):
    visited[cur] = True
    group.append(cur)

    for next in graph[cur]:
        if not visited[next]:
            dfs(next)

    return group

friends = []

for i in range(N):
    if not visited[i]:
        group = []
        friends.append(dfs(i))

ans = 0

for group in friends:
    ans = max(ans,len(group))

# print(friends)
print(ans)