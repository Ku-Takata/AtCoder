N, K = map(int,input().split())
A = list(map(int,input().split()))

# Kと同値の時の処理がめんどい

cnt = 0

for i in range(N-1):
    if A[i] < A[i+1]:
        if A[i] < K < A[i+1]:
            cnt += 1
    else:
        if A[i+1] < K < A[i]:
            cnt += 1

    if A[i+1] == K:
        cnt += 1
    elif A[i] == K and i == 0:
        cnt += 1

print(cnt)