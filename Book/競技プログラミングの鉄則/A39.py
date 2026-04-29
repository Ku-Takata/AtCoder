N = int(input())
LR = [list(map(int,input().split())) for i in range(N)]

# その時その時で終了時刻が早いものを選ぶ
LR.sort(key=lambda x: x[1])
now = 0
cnt = 0

for i in range(N):
    if now <= LR[i][0]:
        now = LR[i][1]
        cnt += 1

print(cnt)