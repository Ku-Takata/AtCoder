X = int(input())

# 普通に普通に2重ループできる

ans = 0

for b in range(1,round(X**0.5)+1):
    for p in range(2,round(X**0.5)+2):
        if b**p > X:
            break
        ans = max(ans,b**p)

print(ans)