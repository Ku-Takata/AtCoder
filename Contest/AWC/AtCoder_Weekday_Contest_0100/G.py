# 各生徒の友達の輪、つまり各連結成分の頂点数を求めたい

import sys
sys.setrecursionlimit(10**7)

def dfs(cur,graph,visited,friend_id,group_id):
    visited[cur] = True
    friend_id[cur] = group_id
    cnt = 1

    for next in graph[cur]:
        if not visited[next]:
            cnt += dfs(next,graph,visited,friend_id,group_id)
    return cnt

N, M = map(int, input().split())
graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)
Q = int(input())
S = [int(input()) for i in range(Q)]

visited = [False] * N
friend_id = [-1] * N
group_size = []
group_id = 0

for i in range(N):
    if not visited[i]:
        size = dfs(i,graph,visited,friend_id,group_id)
        group_size.append(size)
        group_id += 1

for i in range(Q):
    group_id = friend_id[S[i]-1]

    print(group_size[group_id])