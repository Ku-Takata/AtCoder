N = int(input())
A = list(map(int,input().split()))

# 4の倍数がリストに何個あるのか、そして2の倍数は何個あるのかを判定すれば良さそう
# 4の倍数が1つあれば、後ろと前合わせて2つは自由な数で良い
# 2の倍数が2つあれば、リストの最前もしくは最後に連続しておけば良い
# つまり、4の倍数の個数*3 >= リストの要素数-1 - 2の倍数の個数-1
# 2の倍数とそうでない数の間を考慮しないといけない、つまり4の倍数の後に2の倍数が来ないといけないことを考慮して2の倍数の個数-1

cnt_four = 0
cnt_two = 0

for i in range(N):
    if A[i] % 4 == 0:
        cnt_four += 1
    elif A[i] % 2 == 0:
        cnt_two += 1

if cnt_two > 0:
    cnt_two -= 1

if cnt_four*2 >= N - 1 - cnt_two:
    print("Yes")
else:
    print("No")