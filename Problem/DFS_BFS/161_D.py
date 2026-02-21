K = int(input())

import sys
sys.setrecursionlimit(10**7)

runrun = []

def dfs(val):
    if len(runrun) > K:
        return
    if val < 10:
        runrun.append(val)
        return dfs(val+1)

# 1引いた値を最後に付けた数が一番小さく、次に差が同じものが小さく、最後に1足した値の数が小さい ex. 10,11,12
# 例外が存在する ex. 100, 101, 110, 111, 112, 121, 122, 123
# 全部付け終わったら、10の位を1足す
    for n in [abs(val-1), val, abs(val+1)]:
        

    return runrun.append(val)

print(dfs(1))