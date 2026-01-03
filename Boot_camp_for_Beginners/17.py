N = int(input())
V = list(map(int, input().split()))

# 値が小さいものから処理していくと最終的な値が大きくなる
V.sort(reverse=True)

while len(V) >= 2:
    temp = (V[-1] + V[-2]) / 2
    del V[-2:]
    V.append(temp)
    V.sort(reverse=True)

print(*V)