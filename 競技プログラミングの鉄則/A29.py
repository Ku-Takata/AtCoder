a, b = map(int,input().split())

ans = 1
waru = 10**9+7
base = a%waru

while b > 0:
    if b & 1:
        ans = ans*base%waru
    base = base*base%waru
    b >>= 1

print(ans)

# ビットシフトで解いてみた。