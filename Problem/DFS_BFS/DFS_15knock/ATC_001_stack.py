H, W = map(int,input().split())
C = [list(input()) for i in range(H)]

# sからgに行く、#は通れない
"""
import sys
sys.setrecursionlimit(10**7)

def dfs_rec(y,x):
    if x < 0 or x >= W or y < 0 or y >= H:
        return
    elif C[y][x] == "#":
        return
    if visited[y][x]:
        return

    visited[y][x] = True

    dfs_rec(y-1,x)
    dfs_rec(y+1,x)
    dfs_rec(y,x-1)
    dfs_rec(y,x+1)
"""

def dfs_stack(stack):
    while stack:
        sy,sx = stack.pop()

        for dy,dx in [[-1,0],[1,0],[0,-1],[0,1]]:
            ny,nx = sy+dy, sx+dx
            if 0 <= ny < H and 0 <= nx < W and C[ny][nx] != "#" and not visited[ny][nx]:
                visited[ny][nx] = True
                stack.append([ny,nx])

for i in range(H):
    for j in range(W):
        if C[i][j] == "s":
            s_ij = [i,j]
        if C[i][j] == "g":
            g_ij = [i,j]

visited = [[False]*W for i in range(H)]
stack = [[s_ij[0],s_ij[1]]]
# dfs_rec(s_ij[0],s_ij[1])
dfs_stack(stack)

if visited[g_ij[0]][g_ij[1]] == True:
    print("Yes")
else:
    print("No")
