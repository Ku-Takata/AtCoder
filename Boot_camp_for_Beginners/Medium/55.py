N, M = map(int,input().split())
AB = [[] for i in range(N)]

for i in range(M):
    a,b = map(int,input().split())
    a,b = a-1,b-1
    AB[a].append(b)
    AB[b].append(a)

# 逆順で行けるルートがあるか判定したい
# メモ化再帰したい

from functools import lru_cache

@lru_cache(maxsize=200005)
def dfs(n,cnt):
    if n == 0:
        return True
    elif cnt >= 2:
        return
    cnt += 1
    for next in AB[n]:
        if dfs(next,cnt):
            return True

if dfs(N-1,0):
    print("POSSIBLE")
else:
    print("IMPOSSIBLE")