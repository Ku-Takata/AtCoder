N, M = map(int,input().split())

graph = [set() for i in range(N+1)]

# グラフ作成
for i in range(M):
    A, B = map(int,input().split())
    graph[A].add(B)
    graph[B].add(A)

for i in range(1,N+1):
    graph[i] = ", ".join(map(str, sorted(graph[i])))
    print(f"{i}: {{{graph[i]}}}")
