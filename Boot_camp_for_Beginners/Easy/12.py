K, N = map(int, input().split())
A = list(map(int, input().split()))

distance = []

for i in range(N-1):
    distance.append(A[i+1] - A[i])

distance.append(abs(A[0] + (K-A[-1])))

print(sum(distance) - max(distance))