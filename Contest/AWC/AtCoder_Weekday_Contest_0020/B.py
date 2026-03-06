N, M, S = map(int,input().split())
D = list(map(int,input().split()))
PR = [list(map(int,input().split())) for i in range(M)]

PR.sort(key=lambda x: x[0])

tired = False
j = 0

for i in range(N):
    if not tired:
        S -= D[i]
    else:
        S -= D[i]*2

    if S <= 0:
        tired = True

    if j < M and i+1 == PR[j][0]:
        S += PR[j][1]
        j += 1

print(S)