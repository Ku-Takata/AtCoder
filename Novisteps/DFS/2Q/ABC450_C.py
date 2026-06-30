import sys
sys.setrecursionlimit(10**7)
H, W = map(int, input().split())
A = [input() for i in range(H)]

visited = [[False]*W for i in range(H)]
search = [[0,1],[0,-1],[1,0],[-1,0]]

def dfs(cur,status):
    i,j = cur
    visited[i][j] = True

    if i == 0 or i == H-1 or j == 0 or j == W-1:
        status = False

    for y,x in search:
        next = [i+y,j+x]
        if 0 <= next[0] < H and 0 <= next[1] < W:
            if not visited[next[0]][next[1]] and A[next[0]][next[1]] == ".":
                if not dfs(next,status):
                    status = False

    return status

ans = 0

for i in range(H):
    for j in range(W):
        if not visited[i][j] and A[i][j] == ".":
            if dfs([i,j],True):
                ans += 1

print(ans)