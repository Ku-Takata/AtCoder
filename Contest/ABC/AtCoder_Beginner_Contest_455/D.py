N, Q = map(int,input().split())
CP = [list(map(int,input().split())) for i in range(Q)]

# とりあえず図でルールは分かったが、途中で切るのはどうやったらいいんだ感
# ルールは分かるけど実装むずい系か？
# カードPは一番上にある事は保証されているらしいから間に入り込むみたいのはない
# 優先度付キューってこれできるんかな
# なんかできそうな気がしてきた
# 下側から順にpopして乗せるみたいなの
# よく見たら絵はよく見るスタック構造っぽさある
# それぞれでスタック構造を用意してどうにかこうにかかな
# これインデックスの参照の仕方が分からん
# 移動したらそのインデックスをメモしておくみたいなことなんかな、初期は番号通りで

from collections import deque
import heapq
N_que = [deque() for i in range(N)]
for i in range(N):
    N_que[i].append(i+1)
# print(N_que)

for i in range(Q):
    idx_q = N_que.index(CP[i][0])
    