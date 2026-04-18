R, C = map(int,input().split())
sy, sx = map(int,input().split())
gy, gx = map(int,input().split())
maze = [list(input()) for i in range(R)]

sy,sx,gy,gx = sy-1,sx-1,gy-1,gx-1

from collections import deque
import heapq
que = deque()
que.append([sy,sx])
dist = [[False]*C for i in range(R)]
dist[sy][sx] = 0

while que:
    pos = que.popleft()
    if pos[0] == gy and pos[1] == gx:
        break
    for dy,dx in [[-1,0],[1,0],[0,-1],[0,1]]:
        ny,nx = pos[0]+dy, pos[1]+dx
        if 0 <= ny < R and 0 <= nx < C and maze[ny][nx] == "." and dist[ny][nx] == False:
            que.append([ny,nx])
            dist[ny][nx] = dist[pos[0]][pos[1]] + 1

# print(dist)
print(dist[gy][gx])