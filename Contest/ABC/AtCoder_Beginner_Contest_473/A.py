N = int(input())
A = list(map(int,input().split()))

N_2 = N//2
total = 0

for i in range(N_2,N):
    total += A[i]

print(total)