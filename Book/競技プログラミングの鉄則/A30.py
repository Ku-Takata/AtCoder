n, r = map(int,input().split())

import math

ans = math.comb(n,r) % (10**9+7)

print(ans)