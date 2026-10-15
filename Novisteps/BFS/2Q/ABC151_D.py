from collections import deque

H, W = map(int,input().split())
S = [list(input()) for i in range(H)]

# 多始点BFSで最も距離が遠いのがいくつか探す

def bfs_grid_multi(grid, starts):
    h = len(grid)
    w = len(grid[0])

    dist = [[-1] * w for _ in range(h)]
    queue = deque()

    # すべての始点を登録する
    for sy, sx in starts:
        if dist[sy][sx] != -1:
            continue

        dist[sy][sx] = 0
        queue.append((sy, sx))

    directions = [(1, 0), (-1, 0), (0, 1), (0, -1)]

    while queue:
        y, x = queue.popleft()

        for dy, dx in directions:
            ny = y + dy
            nx = x + dx

            # 盤面の外ならスキップ
            if not (0 <= ny < h and 0 <= nx < w):
                continue

            # 壁（"#"）・訪問済みならスキップ
            if grid[ny][nx] == "#" or dist[ny][nx] != -1:
                continue

            dist[ny][nx] = dist[y][x] + 1
            queue.append((ny, nx))

    return dist

ans = 0

for i in range(H):
    for j in range(W):
        if S[i][j] == "#":
            continue

        dist = bfs_grid_multi(S, [(i,j)])

        for row in dist:
            ans = max(ans,max(row))
            # print(row)

        # print("")

print(ans)