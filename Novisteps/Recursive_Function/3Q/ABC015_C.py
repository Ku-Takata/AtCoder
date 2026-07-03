import sys
sys.setrecursionlimit(10**7)
N, K = map(int,input().split())
T = [list(map(int,input().split())) for i in range(N)]

# それぞれ別の値を選んでしまうとバグる

def solve(t,i):
    if i == N:
        if t == 0:
            return 1
        else:
            return 0

    cnt = 0

    for next in T[i]:
        cnt += solve(t^next,i+1)

    return cnt

ans = solve(0,0)

if ans > 0:
    print("Found")
else:
    print("Nothing")