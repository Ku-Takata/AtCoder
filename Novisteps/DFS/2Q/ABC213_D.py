import sys
sys.setrecursionlimit(10**7)
N = int(input())

graph = [[] for i in range(N)]
for i in range(N-1):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

visited = [False] * N
path = []

for i in range(N):
    graph[i].sort()

# print(graph)

def dfs(cur, goal, pre):
    path.append(cur+1)
    if visited[cur]:
        return
    else:
        visited[cur] = True

    for next in graph[cur]:
        if not visited[next]:
            dfs(next,goal,cur)
            path.append(cur+1)

dfs(0, 0, -1)
print(*path)