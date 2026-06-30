import sys
import networkx as nx
sys.setrecursionlimit(10**7)
N, M = map(int, input().split())

edges = []
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    edges.append((u, v))

# 辺の数 - 閉路の辺の数で求められる

graph = nx.Graph()
graph.add_edges_from(edges)

ans = len(list(nx.bridges(graph)))

print(ans)