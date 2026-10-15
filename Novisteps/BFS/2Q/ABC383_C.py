# https://github.com/Ku-Takata/AtCoder/blob/6e7958338cbe03cf691ee113b8381eae3e8f1d19/atcoder_py_snipets.json

from collections import deque

H, W, D = map(int,input().split())
S = [list(input()) for i in range(H)]

dist = [[-1] * W for _ in range(H)]
queue = deque()

for i in range(H):
    for j in range(W):
        if S[i][j] == "H":
            dist[i][j] = 0
            queue.append((i,j))

dir = [(1, 0), (-1, 0), (0, 1), (0, -1)]

while queue:
    i, j = queue.popleft()

    if dist[i][j] == D:
        continue

    for dy, dx in dir:
        ny = i + dy
        nx = j + dx

        if not (0 <= ny < H and 0 <= nx < W):
            continue

        if S[ny][nx] == "#" or dist[ny][nx] != -1:
            continue

        dist[ny][nx] = dist[i][j] + 1
        queue.append((ny, nx))

ans = sum(mass != -1 for row in dist for mass in row)
print(ans)