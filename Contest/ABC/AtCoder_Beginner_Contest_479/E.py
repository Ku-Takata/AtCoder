import sys
sys.setrecursionlimit(10**7)

N = int(input())

# 連結成分が多い方の頂点の値をインデックスとしてxに加算していく問題

graph = [[] for i in range(N)]
for i in range(N-1):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)
    graph[v].append(u)

print(graph)

X = [0]*N

for i in range(N-1):
    