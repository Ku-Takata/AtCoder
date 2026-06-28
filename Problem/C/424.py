N = int(input())
AB = [list(map(int,input().split())) for i in range(N)]

# スキルツリーを作りたい

import sys
sys.setrecursionlimit(10**7)

skill = [[] for i in range(N+1)]

for i in range(1,N+1):
    A,B = AB[i-1]

    skill[A].append(i)
    skill[B].append(i)

ok = [0]*(N+1)
ok[0] = 1

def dfs(v):
    ok[v] = 1
    for next in skill[v]:
        if not ok[next]:
            dfs(next)

dfs(0)

print(sum(ok)-1)