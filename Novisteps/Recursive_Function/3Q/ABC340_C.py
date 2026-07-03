import sys
sys.setrecursionlimit(10**7)

N = int(input())

from functools import lru_cache

@lru_cache(maxsize=1000)
def solve(x):
    if x < 2:
        return 0

    num1,num2 = x//2,-(-x//2)
    money = x

    money += solve(num1)
    money += solve(num2)

    return money

ans = solve(N)
print(ans)