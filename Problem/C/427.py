N, M = map(int, input().split())

# 隣り合う頂点の色が別々になっているかどうかを判定する
# 削除する辺は全探索でも可能？
# 2部グラフかどうかの判定はO(V+E)

graph = [[] for i in range(N)]
edges = []

for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)
    edges.append((u, v))

min_remove_edges = M

import itertools

for color in itertools.product([0, 1], repeat=N):
    current_remove_count = 0

    for u, v in edges:
        if color[u] == color[v]:
            current_remove_count += 1

    if current_remove_count < min_remove_edges:
        min_remove_edges = current_remove_count

print(min_remove_edges)