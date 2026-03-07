N, M = map(int,input().split())
C = list(map(int,input().split()))

total = 0

for i in range(N):
    A, B = map(int,input().split())

    if C[A-1] >= B:
        C[A-1] -= B
        total += B
    else:
        total += C[A-1]
        C[A-1] = 0

print(total)