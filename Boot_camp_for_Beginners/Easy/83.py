N = int(input())
A = [list(map(int,input().split())) for i in range(2)]

ans = 0

for i in range(N):
    candy = sum(A[0][0:i+1])
    for j in range(i,N):
        candy += A[1][j]

    ans = max(ans,candy)

print(ans)