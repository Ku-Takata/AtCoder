N, M = map(int,input().split())
B = list(map(int,input().split()))

graph = [[] for i in range(N)]
for i in range(M):
    u, v = map(int, input().split())
    u, v = u - 1, v - 1
    graph[u].append(v)

print(graph)

# これは...たぶん解けないなあ
# 実装頑張らないといけない系だから苦手問題
# どうやったらいいかは何となく検討がつく