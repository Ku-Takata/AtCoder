N, M = map(int,input().split())

# 裏返されるのが奇数回で裏、偶数で表になる
# 制約がN,M共に10**9までなので何らかの規則性があると考えられる
# 自分のマスと周りのマスの数の和の偶奇を判定すれば分かる
# 自分のマス + 周りのマスの数 = 偶数なら表、奇数なら裏
# 角と端は確定で表になり、それ以外は確定で裏になる

if N >= 2 and M >= 2:
    front = 2*N + 2*M -4
elif N == 1 and M == 1:
    front = 0
else:
    front = 2

back = N*M - front

print(back)