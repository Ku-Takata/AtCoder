# 計算量はN * P(最小座標から最大座標まで)
"""
N = int(input())
X = list(map(int, input().split()))
result_list = []

for i in range(min(X), max(X)+1):
    result = 0
    for x in X:
        temp = (x-i)**2
        result += temp
    result_list.append(result)

print(min(result_list))
"""

# 計算量をNにする
N = int(input)
X = list(map(int, input().split()))

avg = sum(X) / N
p = int(avg + 0.5) if avg > 0 else int(avg - 0.5)

ans = sum((x-p)**2 for x in X)
print(ans)