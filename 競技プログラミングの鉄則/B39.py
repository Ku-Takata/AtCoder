N, D = map(int,input().split())
XY = [list(map(int,input().split())) for i in range(N)]

XY.sort(key=lambda x: x[0])
ans = 0
money = []
j = 0
# print(XY)
# それぞれの日で最も報酬が良いのを選び続ける、仕事はそれぞれ1回しかできない
for i in range(1,D+1):
    while j < N and XY[j][0] <= i:
        money.append(XY[j][1])
        j += 1

    if money:
        work = max(money)
        ans += work
        money.remove(work)

print(ans)

# テストケースが優しいおかげで、PyPyで全ケースACになったが、基本的にはこれだと部分点までしか取れない