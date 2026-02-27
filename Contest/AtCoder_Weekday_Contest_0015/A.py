A = list(map(int,input().split()))
B = list(map(int,input().split()))

ans = 0
for i in range(len(A)):
    ans += A[i]*B[i]

print(ans)