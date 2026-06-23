ABC = list(map(int,input().split()))

# できる限り体積が同じになるように直方体を作りたい
# 一番長い辺のところで2分割したら良さそう

import math

ABC.sort()
block_X = (ABC[2]//2)*ABC[0]*ABC[1]
block_Y = math.prod(ABC) - block_X

print(abs(block_X-block_Y))