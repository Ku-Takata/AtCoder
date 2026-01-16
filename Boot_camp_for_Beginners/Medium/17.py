N,T = map(int,input().split())
t = list(map(int,input().split()))

total = T

for i in range(1,N):
    if t[i] - t[i-1] >= T:
        total += T
    else:
        total += t[i] - t[i-1]

print(total)