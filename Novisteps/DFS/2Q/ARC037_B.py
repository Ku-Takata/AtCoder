import sys
sys.setrecursionlimit(10**7)
N, M = map(int, input().split())

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

# 閉路、つまりループしない木の数をカウントする

visited = [False for i in range(N)]

def dfs(cur,pre):
    visited[cur] = True
    status = True

    for next in graph[cur]:
        if next == pre:
            continue
        if visited[next]:
            status = False
        else:
            if not dfs(next,cur):
                status = False

    return status

ans = 0

for i in range(N):
    if not visited[i]:
        if dfs(i,-1):
            ans += 1

print(ans)