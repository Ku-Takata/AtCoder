N, K = map(int,input().split())
A = list(map(int,input().split()))

takahashi = 0
aoki = 0

for i in range(N):
    if takahashi+A[i] <= K:
        takahashi += A[i]
    else:
        aoki += A[i]

if takahashi > aoki:
    print("Takahashi")
elif aoki > takahashi:
    print("Aoki")
else:
    print("Draw")