N = int(input())
A = list(map(int,input().split()))

ans = float("-inf")

for i in range(N):
    min_mikan = A[i]
    for j in range(i,N):
        min_mikan = min(min_mikan, A[j])
        total = min_mikan*(j-i+1)
        if total > ans:
            ans = total

print(ans)