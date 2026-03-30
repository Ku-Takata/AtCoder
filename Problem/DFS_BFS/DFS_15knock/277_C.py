N = int(input())

from collections import defaultdict
import sys
sys.setrecursionlimit(10**7)

graph = defaultdict(list)
visited = dict()

for i in range(N):
    A, B = map(int,input().split())
    graph[A].append(B)
    graph[B].append(A)
    visited[A] = False
    visited[B] = False

ans = 1

def dfs(root):
    global ans
    if visited[root]:
        return
    visited[root] = True
    if root > ans:
        ans = root

    for next_node in graph[root]:
        dfs(next_node)

if graph[1]:
    dfs(1)
print(ans)