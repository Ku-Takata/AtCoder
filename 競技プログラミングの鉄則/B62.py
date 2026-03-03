import sys
sys.setrecursionlimit(10**7)

def dfs(pos):
    # 経路を記録
    visited[pos] = True
    path.append(pos)
    # Nにたどり着いたらTrueで返す
    if pos == N:
        return True

    # 行っていない経路かつNにたどり着くのであればTrueを返す
    for i in graph[pos]:
        if not visited[i]:
            if dfs(i):
                return True

    # 行き止まりかつNでないなら経路から削除
    path.pop()
    return False

N, M = map(int,input().split())

graph = [[] for i in range(N+1)]
for i in range(M):
    A, B = map(int,input().split())
    graph[A].append(B)
    graph[B].append(A)

visited = [False]*(N+1)
path = []

if dfs(1):
    print(*path)
else:
    print(-1)