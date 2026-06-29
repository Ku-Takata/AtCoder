import sys
sys.setrecursionlimit(10**7)

H, W = map(int,input().split())
S = [input() for i in range(H)]

# 連結成分を求める問題

visited = [[False]*W for i in range(H)]

def dfs(cur):
    y,x = cur
    visited[y][x] = True

    for i in [-1,0,1]:
        for j in [-1,0,1]:
            next = [y+i,x+j]
            if 0 <= next[0] < H and 0 <= next[1] < W:
                if not visited[next[0]][next[1]] and S[next[0]][next[1]] == "#":
                    dfs(next)

    return True

ans = 0

for i in range(H):
    for j in range(W):
        if not visited[i][j] and S[i][j] == "#":
            dfs([i,j])
            ans += 1

print(ans)