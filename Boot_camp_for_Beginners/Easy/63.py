N = int(input())
A = list(map(int,input().split()))

num = 1
cnt = 0

for i in range(N):
    if A[i] == num:
        num += 1
    else:
        cnt += 1

if cnt == N:
    print(-1)
else:
    print(cnt)