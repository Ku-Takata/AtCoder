N, M, X = map(int, input().split())
A = list(map(int, input().split())) #コストがかかるマス

count1 = 0
count2 = 0

# Nから0へ目指すとき
for i in range(X):
    if i == 0:
        continue
    elif X-i in A:
        count1 += 1
    else:
        continue

# Nからゴールを目指すとき
for j in range(N-X):
    if j == 0:
        continue
    elif X+j in A:
        count2 += 1
    else:
        continue

if count1 > count2:
    print(count2)
else:
    print(count1)