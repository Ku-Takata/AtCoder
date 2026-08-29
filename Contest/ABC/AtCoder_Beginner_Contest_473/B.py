N = int(input())
A = list(map(int,input().split()))

for i in range(N-1):
    for j in range(i+1,N):
        if A[i] == A[j]:
            A[i] = 0
            A[j] = 0

print(sum(A))