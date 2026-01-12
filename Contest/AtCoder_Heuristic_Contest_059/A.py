N = int(input())
A = []
now = [[0,0]]

for i in range(N):
    A.append(list(map(int, input().split())))

# 目的は最小ステップで、盤面からカードを無くすこと

# 愚直バージョン(0から順番に全探索して2枚集めていく)
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


# 以下入力で54ターン
"""
4
2 4 0 3
5 2 5 1
1 4 0 7
3 7 6 6

"""