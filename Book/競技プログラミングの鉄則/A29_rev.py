a, b = map(int,input().split())

ans = 1
waru = 10**9+7

for i in range(30):
    binary = 2**i
    if (b//binary) % 2 == 1:
        ans = (ans*a) % waru
    a = (a*a) % waru

print(ans)

# 著者のコード参考