from functools import lru_cache

N = int(input())

@lru_cache(maxsize=None)
def solve(n):
    if n == 1:
        return [1]
    else:
        return solve(n-1) + [n] + solve(n-1)

print(*solve(N))