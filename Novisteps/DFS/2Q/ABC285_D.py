import sys
sys.setrecursionlimit(10**7)
N = int(input())

graph = dict()
state = dict()
for i in range(N):
    u, v = map(str, input().split())
    graph[u] = v
    state[u] = 0
    state[v] = 0

# サイクルグラフがあったらNo、1つもなかったらYes
# 文字列のグラフってどう作るんだ？

# 未探索0、探索中1、探索済み2にする

def dfs(cur,is_cycle):
    state[cur] = 1
    if cur not in graph:
        return False
    next = graph[cur]

    if state[next] == 0:
        if dfs(next,is_cycle):
            is_cycle = True
    elif state[next] == 1:
        is_cycle = True

    state[cur] = 2

    return is_cycle

for u,v in graph.items():
    if state[u] == 0:
        if dfs(u,False):
            print("No")
            exit()

print("Yes")