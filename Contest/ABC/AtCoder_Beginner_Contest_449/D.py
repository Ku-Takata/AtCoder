L, R, D, U = map(int,input().split())

# xがある値のとき、yの最小の値と比べてどのくらいの差があるのかを計算
# それをxの取れる値の範囲だけ分やればできそう
# というか図が大ヒントで、図の通りにやればよさそう
# y = 奇数でx>yの範囲で交互に黒、x<yの範囲で白
# y = 偶数でx>yの範囲で交互に黒、x<yの範囲で黒

# yを中心で考える
import math

ans = 0

for y in range(D,U+1):
    if y % 2 == 1:
        ans += math.ceil((abs(L)-abs(y)) / 2)
        ans += math.ceil((abs(R)-abs(y)) / 2)
    else:
        if abs(y) <= abs(L):
            ans += (abs(L)-abs(y)) // 2 + abs(y)
        else:
            ans += (abs(L)-abs(y)) // 2 + abs(abs(y)-abs(D))
        if abs(y) <= abs(R):
            ans += (abs(R)-abs(y)) // 2 + abs(y)
        else:
            ans += (abs(L)-abs(y)) // 2 + abs(abs(y)-abs(U))
        ans += 1

print(ans)

# 違ったけど多分普通に解ける、時間はかかりそうだが