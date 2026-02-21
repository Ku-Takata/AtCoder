N = int(input())

import sys
sys.setrecursionlimit(10**7)

def dfs(current_val):
    if current_val > N:
        return 0

    s = str(current_val)
    if "3" in s and "5" in s and "7" in s:
        res = 1
    else:
        res = 0

    for i in [3,5,7]:
        res += dfs(current_val*10+i)

    return res

print(dfs(0))