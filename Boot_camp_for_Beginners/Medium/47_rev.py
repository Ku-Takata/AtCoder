N, P = map(int, input().split())
A = list(map(int, input().split()))

# 奇数の数をカウントする
cnt_odd = sum(1 for x in A if x % 2 != 0)

if cnt_odd == 0:
    # すべて偶数の場合
    if P == 0:
        ans = 2 ** N
    else:
        ans = 0
else:
    # 奇数が1つでもあれば、偶数にする方法と奇数にする方法は半分ずつ
    ans = 2 ** (N - 1)

print(ans)