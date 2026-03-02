def dfs(pos):
    print(pos, end=' ')
    for i in graph[pos]:
        dfs(i)

N, M = map(int,input().split())
graph = [[] for i in range(N+1)]

for i in range(M):
    A, B = map(int,input().split())
    graph[A].append(B)
    graph[B].append(A)

dfs(0)

# 明日はここからやる