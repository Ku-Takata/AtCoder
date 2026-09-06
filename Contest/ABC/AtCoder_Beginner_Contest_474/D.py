N = int(input())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

# つまりAとBの各要素*任意のWの総和の差を最大化したい
# Aの要素が勝っていたならWiを10**18、負けていたら1にする
# 10**18の石1つでも多く持っていたらYesでは？
# 仮に1つだけ多く持っていて後はBが最大値石を持っていたとすると、10**9 * 10**5 = 10**14なので、勝てないはず

W = []
total = 0
mod = 0

for i in range(N):
    if A[i] > B[i]:
        W.append(10**18)
        total += A[i]-B[i]
    else:
        W.append(1)
        mod += B[i]-A[i]

if total > 0:
    print("Yes")
    print(*W)
else:
    print("No")