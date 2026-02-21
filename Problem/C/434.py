T = int(input())
for i in range(T):
    N, H = map(int,input().split())

    ans = "Yes"
    move_T = 0
    tlv = []

    for j in range(N):
        tlv.append(list(map(int,input().split())))

    for j in range(N):
        dt = tlv[j][0]-move_T
        if j == 0:
            max_H = H+dt
            min_H = H-dt
        else:
            max_H = max_H + dt
            min_H = min_H - dt

        max_H = min(max_H,tlv[j][2])
        min_H = max(min_H,tlv[j][1])
        move_T = tlv[j][0]

        if min_H > max_H:
            ans = "No"

    print(ans)

# やる事は分かったが、実装が大変だった。実装大変系の問題は苦手なので精進したいところ。
# Greedy algo, interval update