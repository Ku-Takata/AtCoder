import sys
sys.setrecursionlimit(10**7)

def dfs(pos):
    if visited[pos]:
        return
    else:
        # print(pos, end=' ')
        visited[pos] = True
    for i in graph[pos]:
        dfs(i)

N, M = map(int,input().split())
graph = [[] for i in range(N+1)]

for i in range(M):
    A, B = map(int,input().split())
    graph[A].append(B)
    graph[B].append(A)

visited = [False]*(N+1)
dfs(1)

for i in range(1,N+1):
    if visited[i]:
        continue
    else:
        print("The graph is not connected.")
        exit()

print("The graph is connected.")