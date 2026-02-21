N, Q = map(int,input().split())
A = list(map(int,input().split()))
LR = []
for i in range(Q):
    LR.append(list(map(int,input().split())))

cs = []

for i in range(N):
    if i == 0:
        cs.append(A[i])
    else:
        cs.append(cs[i-1]+A[i])

for i in range(Q):
    if LR[i][0] == 1:
        print(cs[LR[i][1]-1])
    else:
        print(cs[LR[i][1]-1] - cs[LR[i][0]-2])