import sys
sys.setrecursionlimit(10**7)
N, X, Y = map(int, input().split())
X -= 1
Y -= 1

graph = [[] for i in range(N)]
for i in range(N-1):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

# print(graph)

# 行き止まりかつYでなければ戻る

visited = [False] * N
path = []

def dfs(cur):
    visited[cur] = True
    path.append(cur+1)

    if cur == Y:
        return True

    for next in graph[cur]:
        if visited[next]:
            continue
        if dfs(next):
            return True

    path.pop()

    return False

dfs(X)
print(*path)