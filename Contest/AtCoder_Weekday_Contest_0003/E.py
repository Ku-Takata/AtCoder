N, M = map(int,input().split())
W = sorted(list(map(int,input().split())),reverse=True)
C = sorted(list(map(int,input().split())),reverse=True)

for i in range(M):
    total = 0
    while len(W) != 0:
        if total + W[0] <= C[i]:
            total += W[0]
            W.pop(0)
        else:
            break
    while len(W) != 0:
        if total + W[-1] <= C[i]:
            total += W[-1]
            W.pop(-1)
        else:
            break

if len(W) == 0:
    print("Yes")
else:
    print("No")