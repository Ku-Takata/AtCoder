H, W = map(int,input().split())

# 移動回数はH+W-2,右移動回数はW-1
move = H+W-2
right = W-1

import math

ans = math.comb(move,right) % (10**9+7)

print(ans)