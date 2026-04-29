N, M = map(int,input().split())
graph = [set() for i in range(N+1)]

for i in range(M):
    A, B = map(int,input().split())
    graph[A].add(B)
    graph[B].add(A)

friends = 0

for i in range(1,N+1):
    if len(graph[i]) > friends:
        friends = len(graph[i])
        num = i

print(num)