from collections import deque
import sys
sys.setrecursionlimit(10**7)

N, M, S, T = map(int,input().split())

# ルートをDFSかBFSで全探索して、その中で燃料が最小のものを探す
# これ01BFSか
# 参考URL: https://qiita.com/manuo/items/e5ed45aae401ccc41afd#01bfs-2

S,T = S-1,T-1

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)

# print(graph)

memo = [-1]*N
memo[S] = 0

free = []

for edge in graph:
    if edge:
        free.append(min(edge))
    else:
        free.append(-1)

# print(free)

q = deque([S])
while q:
    # print(q)
    # print(memo)
    now = q.popleft()

    for next in graph[now]:
        cost = 0
        if next != free[now]:
            cost = 1

        total = memo[now] + cost

        if memo[next] == -1 or total < memo[next]:
            memo[next] = total

            if cost == 0:
                q.appendleft(next)
            else:
                q.append(next)

print(memo[T])