N, Q = map(int,input().split())
C = list(map(int,input().split()))

renzoku = [0]

# 連続した数を新たに累積和でリスト化する
for i in range(1,N):
    if C[i] == C[i-1]:
        renzoku.append(renzoku[-1]+1)
    else:
        renzoku.append(renzoku[-1])

# print(renzoku)

LR = [list(map(int,input().split())) for i in range(Q)]

for i in range(Q):
    ans = renzoku[LR[i][1]-1] - renzoku[LR[i][0]-1]
    print(ans)