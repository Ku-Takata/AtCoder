H, W = map(int,input().split())
C = [list(input()) for i in range(H)]

# sからgに行く、#は通れない

import sys
sys.setrecursionlimit(10**7)

def dfs(y,x):
    if x < 0 or x >= W or y < 0 or y >= H:
        return
    elif C[y][x] == "#":
        return
    if visited[y][x]:
        return

    visited[y][x] = True

    dfs(y-1,x)
    dfs(y+1,x)
    dfs(y,x-1)
    dfs(y,x+1)

visited = [[False]*W for i in range(H)]

for i in range(H):
    for j in range(W):
        if C[i][j] == "s":
            s_ij = [i,j]
        if C[i][j] == "g":
            g_ij = [i,j]

dfs(s_ij[0],s_ij[1])

if visited[g_ij[0]][g_ij[1]] == True:
    print("Yes")
else:
    print("No")

# MLEが1つ出る。再帰だと起こる問題。