N = int(input())

# これ途中で10**9 + 7で割ってもいいやつだよね

power = 1

for i in range(N):
    power *= i+1
    power %= 10**9+7

print(power)