N = int(input())
A = list(map(int,input().split()))

ans = "Yes"

for i in range(N):
    if A[i] >= 0:
        ans = "No"
        break

print(ans)