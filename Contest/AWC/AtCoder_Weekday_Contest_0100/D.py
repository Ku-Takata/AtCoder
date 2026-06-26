N = int(input())
A = list(map(int,input().split()))

# チーム差がないようにしたい

L = 0
R = sum(A)
ans = float("inf")

for i in range(N):
    L += A[i]
    R -= A[i]

    if abs(L-R) < ans:
        ans = abs(L-R)

print(ans)