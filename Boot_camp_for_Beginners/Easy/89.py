X, Y = map(int,input().split())

# Xを2倍し続けてYを越えたらお終いにして長さを出力でいいと思う

ans = 1

while X*2 <= Y:
    X *= 2
    ans += 1

print(ans)