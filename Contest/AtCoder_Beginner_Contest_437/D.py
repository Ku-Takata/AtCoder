N, M = map(int, input().split())
waru = int(998244353)

list_result = []

A = list(map(int, input().split()))
B = list(map(int, input().split()))

for a in A:
    for b in B:
        result = abs(a - b)
        list_result.append(result)

total = sum(list_result)

print(total % waru)

# 15/32 TLE forループ2回だとO(2NM)ぐらいある