N, M, C = map(int, input().split())
B = list(map(int, input().split()))
A = []
result = []
count = 0

for i in range(N):
    A.append(list(map(int, input().split())))

for i in range(N):
    for m in range(M):
        result.append(A[i][m] * B[m])
    if sum(result) + C > 0:
            count += 1
    result.clear()

print(count)