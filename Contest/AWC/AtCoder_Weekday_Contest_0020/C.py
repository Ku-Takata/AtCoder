N = int(input())
V = list(map(int,input().split()))

V = sorted(V)
total = 0

for i in range(N-1):
    total += V[i+1]-V[i]

print(total)