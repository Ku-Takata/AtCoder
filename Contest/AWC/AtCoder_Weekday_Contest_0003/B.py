N = int(input())
LR = []
for i in range(N):
    LR.append(list(map(str,input().split())))

cnt = 0

for i in range(N-1):
    if LR[i][1] == LR[i+1][0]:
        cnt += 1

print(cnt)