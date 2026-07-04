import sys

T = int(sys.stdin.readline())

# XもYもKで割り続けるといつか0に収束する
# 一旦どちらも割り続けて、それを記録して、初めて同じ値になったところが答え

for i in range(T):
    X, Y, K = map(int,sys.stdin.readline().split())

    if X == Y:
        print(0)
        continue

    calc_x = {}
    cur = X
    cnt_x = 0

    # Xから0になるまでKで割る
    while True:
        calc_x[cur] = cnt_x
        if cur == 0:
            break
        cur //= K
        cnt_x += 1

    # print(calc_x)

    ans = float("inf")
    cur = Y
    cnt_y = 0

    # Yから0になるまでKで割りながら、XとYが同じになる値を見つける
    while True:
        if cur in calc_x:
            cnt_total = calc_x[cur] + cnt_y
            ans = min(ans, cnt_total)
            break
        if cur == 0:
            break
        cur //= K
        cnt_y += 1

    print(ans)