import itertools
import math
import sys
sys.setrecursionlimit(10**7)

N, X = map(int,input().split())
LA = [list(map(int,input().split())) for i in range(N)]

def solve(a,i):
    if i == N:
        if a == X:
            return 1
        else:
            return 0

    cnt = 0

    for next in LA[i][1:]:
        cnt += solve(a*next,i+1)

    return cnt

ans = solve(1,0)

print(ans)