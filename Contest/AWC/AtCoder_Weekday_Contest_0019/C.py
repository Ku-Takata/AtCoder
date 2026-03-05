N = int(input())
A = list(map(int,input().split()))

A = sorted(A)
cnt = 1

for i in range(N-1):
    if A[i]+1 != A[i+1]:
        cnt += 1

print(cnt)