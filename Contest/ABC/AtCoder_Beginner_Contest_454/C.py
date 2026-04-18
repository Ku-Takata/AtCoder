N, M = map(int,input().split())
AB = [list(map(int,input().split())) for i in range(M)]

# グラフの到達可能数を数える問題
# グラフは有効グラフ
# 戻っても別に良くて、ループはダメというようにしたい

graphs = [[] for i in range(N+1)]

for i in range(M):
    graphs[AB[i][0]].append(AB[i][1])

from collections import deque
import heapq
Q = deque([1])
visited = [False]*(N+1)
visited[1] = True
cnt = 1

while Q:
    pos = Q.popleft()

    for next in graphs[pos]:
        if not visited[next]:
            visited[next] = True
            cnt += 1
            Q.append(next)

print(cnt)