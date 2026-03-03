N, M = map(int,input().split())
S, T = map(int,input().split())
PV = [list(map(int,input().split())) for i in range(M)]

ans = 0

if S < T:
    for i in range(M):
        if PV[i][0] < S:
            continue
        elif PV[i][0] > T:
            continue
        else:
            ans += PV[i][1]
else:
    for i in range(M):
        if PV[i][0] < T:
            continue
        elif PV[i][0] > S:
            continue
        else:
            ans += PV[i][1]

print(ans)