H, W = map(int,input().split())
C = [list(input()) for i in range(H)]

import sys
sys.setrecursionlimit(10**7)

def dfs_rec(sy,sx):
    visited[sy][sx] = True

    for dy,dx in [[-1,0],[1,0],[0,-1],[0,1]]:
        ny,nx = sy+dy, sx+dx
        if 0 <= ny < H and 0 <= nx < W and C[ny][nx] != "#" and not visited[ny][nx]:
            dfs_rec(ny,nx)

for i in range(H):
    for j in range(W):
        if C[i][j] == "s":
            s_idx = [i,j]
        if C[i][j] == "g":
            g_idx = [i,j]

visited = [[False]*W for i in range(H)]
dfs_rec(s_idx[0],s_idx[1])

if visited[g_idx[0]][g_idx[1]] == True:
    print("Yes")
else:
    print("No")