N, K = map(int,input().split())

if N < K:
    print(1)
    exit()

A = [1] * K
total = sum(A[0:K])

for i in range(K-1,N):
    total = total + A[i] - A[i-K]
    A.append(total % 10**9)

print(A[-1])