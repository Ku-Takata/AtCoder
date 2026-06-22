N = int(input())
town = [list(map(int,input().split())) for i in range(N)]
# 全通りを試して平均取る

from itertools import permutations
import math

ans = 0

for p in permutations(town):
    # print(p)
    for i in range(N-1):
        calc = (p[i][0]-p[i+1][0])**2 + (p[i][1]-p[i+1][1])**2
        ans += calc**0.5

ans = ans/math.factorial(N)
print(ans)