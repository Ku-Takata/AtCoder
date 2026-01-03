N = int(input())
D = list(map(int, input().split()))

sort_D = sorted(D)

middle_plus1 = (len(sort_D) // 2)

if N % 2 == 1:
    print(0)
else:
    print(sort_D[middle_plus1] - sort_D[middle_plus1 - 1])

# 真ん中2つの差の値分、Kが存在する