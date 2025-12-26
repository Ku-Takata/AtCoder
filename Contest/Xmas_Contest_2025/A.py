K = int(input())
A = list(map(int, input().split()))
B = list(map(int, input().split()))
C = list(map(int, input().split()))

result = []

# K倍するとこうなる
for i in range(200*K):
    for j in range(200*K):
        kakudaimae_i = i//K
        kakudaimae_j = j//K
        A[i][j] = A[kakudaimae_i][kakudaimae_j]
        B[i][j] = B[kakudaimae_i][kakudaimae_j]
        C[i][j] = C[kakudaimae_i][kakudaimae_j]

print((A[i] * B[j] * C[i + j]) % 998244353)