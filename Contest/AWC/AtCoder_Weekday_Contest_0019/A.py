N, K = map(int,input().split())
A = list(map(int,input().split()))

cnt = 0

for i in range(N):
    if A[i] >= K:
        cnt += 1

print(cnt)