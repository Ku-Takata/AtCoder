N = int(input())
A = []

for i in range(N):
    A.append(int(input()))

left = [0] * (N+1)
right = [0] * (N+1)

for i in range(N-1):
    left[i+1] = max(A[i], left[i])

for i in range(N-1, -1, -1):
    right[i] = max(A[i], right[i+1])

for i in range(N):
    ans = max(left[i],right[i+1])
    print(ans)