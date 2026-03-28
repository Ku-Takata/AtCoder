H, W = map(int,input().split())
C = [list(input()) for i in range(H)]

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
            s_idx = [i,j]
        if C[i][j] == "g":
            g_idx = [i,j]

stack = [[s_idx[0],s_idx[1]]]
visited = [[False]*W for i in range(H)]

dfs_stack(stack)

if visited[g_idx[0]][g_idx[1]] == True:
    print("Yes")
else:
    print("No")