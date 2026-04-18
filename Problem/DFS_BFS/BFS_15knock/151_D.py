H, W = map(int,input().split())
S = [list(input()) for i in range(H)]

# 方針としてはスタート位置を全探索し、BFSで最大距離のゴール位置を調べる
# 一番大きい値を取れるスタート位置、ゴール位置が答え

from collections import deque
import numpy as np

ans = 0

for i in range(H):
    for j in range(W):
        if S[i][j] == "#":
            continue

        dist = [[0]*W for i in range(H)]
        Q = deque([[i,j]])

        while Q:
            sy,sx = Q.popleft()
            for dy,dx in [[-1,0],[1,0],[0,-1],[0,1]]:
                ny,nx = sy+dy, sx+dx
                if 0 <= ny < H and 0 <= nx < W and S[ny][nx] == "." and dist[ny][nx] == 0:
                    dist[ny][nx] = dist[sy][sx] + 1
                    Q.append([ny,nx])

        dist[i][j] = 0
        ans = max(ans,np.max(dist))

print(ans)