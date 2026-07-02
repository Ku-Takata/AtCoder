import sys
import networkx as nx
sys.setrecursionlimit(10**7)
N, M = map(int, input().split())

graph = [[] for i in range(N)]
edge = []
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)
    edge.append((u, v))

# パスグラフかつ特定の生徒が横になるようにできるかチェック
# 1人の生徒に3人横に並びたいとなっていると不可能
# 2人は1人だけ横に並びたいとなっていないと不可能

def cnt_cycle_edges():
    G = nx.Graph()
    G.add_edges_from(edge)

    ans = M - len(list(nx.bridges(G)))
    return ans

if cnt_cycle_edges() != 0:
    print("No")
    exit()

# print(graph)
cnt = 0

for request in graph:
    request = set(request)
    if len(request) > 2:
        print("No")
        exit()
    elif len(request) <= 1:
        cnt += 1

if cnt >= 2:
    print("Yes")
else:
    print("No")