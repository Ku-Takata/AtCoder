N, K = map(int,input().split())
A = list(map(int,input().split()))

j = 0
cnt = 0

for i in range(N):
    left = A[i]
    while j < N-1:
        if A[j+1]-left <= K:
            j += 1
        else:
            break

    cnt += j-i

print(cnt)