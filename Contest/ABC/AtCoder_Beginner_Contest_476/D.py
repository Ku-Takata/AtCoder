from bisect import bisect_right

N, M, K = map(int, input().split())
X, Y = map(int, input().split())
A = sorted(map(int, input().split()))
B = sorted(map(int, input().split()))

# BはKドル札でしか支払いができない
# 1ドル札でAの中で安いのから買っていく
# 次の安いのを買うのに1ドル札が足りなくなったらKドル札でBの安いやつを買う or Aのを買う
# ここでどちらを買った方が種類数が多くなるかを判定する必要が出てくる
# それ難しすぎない？？
# 違うな、BだけKドル札しか使えないならBから安いのを1個ずつ買う個数を増やしたときに最大値が変わるかどうかを全探索したらいい

cs_A = [0]
for a in A:
    cs_A.append(cs_A[-1]+a)

ans = 0
B_price = 0
K_cnt = 0

for i in range(M+1):
    if K_cnt > Y:
        break

    money = X + Y*K - B_price
    B_cnt = bisect_right(cs_A,money) - 1
    ans = max(ans, i+B_cnt)

    if i == M:
        break

    B_price += B[i]
    K_cnt += (B[i] + K-1) // K

print(ans)