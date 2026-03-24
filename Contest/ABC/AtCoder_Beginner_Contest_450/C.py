H, W = map(int,input().split())
S = [list(str(input())) for i in range(H)]

# 外側の枠に居ないかつ、隣に必ず１つ以上白がある状態の塊をカウントする
# 外側の枠でないかつ、途切れたらカウント？
# 幅優先みたいなことしたら解けそう
# bfsまだ実装あんまりできないから解けなさそう...

from collections import deque
import heapq

visited = [[False]*W for i in range(H)]

def bfs()