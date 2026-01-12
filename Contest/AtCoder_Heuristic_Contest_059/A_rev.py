N = int(input())
A = []
now = [[0,0]]
cnt = 0

for i in range(N):
    A.append(list(map(int, input().split())))

def move_calc(i,j,now):
    sayuu = now[0][1] - j
    if sayuu < 0:
        for k in range(abs(sayuu)):
            print("R")
    elif sayuu > 0:
        for k in range(abs(sayuu)):
            print("L")

    zyouge = now[0][0] - i
    if zyouge < 0:
        for k in range(abs(zyouge)):
            print("D")
    elif zyouge > 0:
        for k in range(abs(zyouge)):
            print("U")

    now = [[i,j]]
    print("Z")
    print(now)
    return now

# A[now[0][0]][now[0][1]]
def search_same(n):
    for i in range(N):
        for j in range(N):
            if n == A[i][j]:
                move_calc(i,j)


# 少しスマートにする(近くにカードがあるところから拾って、同じカードを見つける)
while cnt != (N**2 // 2):
    for n_near in range(N):
        #カードがある条件
        # 右確認、あるなら移動 & Z & continue
        if now[0][1] < N-1 and A[now[0][0]][now[0][1]+n_near] >= 0:
            move_calc(now[0][0], now[0][1]+n_near, now)
            # カードを取った所を-1にして空のマスを表現
            print(A[now[0][0]][now[0][1]+n_near])
            n = A[now[0][0]][now[0][1]+n_near]
            A[now[0][0]][now[0][1]+n_near] == -1
            search_same(n)
            print(A)
        # 下
        elif now[0][0] < N-1 and A[now[0][0]+n_near][now[0][1]] >= 0:
            move_calc(now[0][0]+n_near, now[0][1], now)
            A[now[0][0]+n_near][now[0][1]] == -1
        # 左
        elif now[0][1] > 0 and A[now[0][0]][now[0][1]-n_near] >= 0:
            move_calc(now[0][0], now[0][1]-n_near), now
            A[now[0][0]][now[0][1]-n_near] == -1
        # 上
        elif now[0][0] > 0 and A[now[0][0]-n_near][now[0][1]] >= 0:
            move_calc(now[0][0]-n_near, now[0][1], now)
            A[now[0][0]-n_near][now[0][1]] == -1



    cnt += 1

# 貪欲法での実装ができなかった。重めの実装に対し、惑わされずに実装できるようになりたい。




"""
for cn in range(N**2 // 2):
    for i in range(N):
        for j in range(N):
            if cn == A[i][j]:
                sayuu = now[0][1] - j
                if sayuu < 0:
                    for k in range(abs(sayuu)):
                        print("R")
                elif sayuu > 0:
                    for k in range(abs(sayuu)):
                        print("L")

                zyouge = now[0][0] - i
                if zyouge < 0:
                    for k in range(abs(zyouge)):
                        print("D")
                elif zyouge > 0:
                    for k in range(abs(zyouge)):
                        print("U")

                now = [[i,j]]
                print("Z")
                # print(now)

"""