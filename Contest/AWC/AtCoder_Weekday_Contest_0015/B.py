N, K = map(int,input().split())
P = list(map(int,input().split()))

for i in range(N):
    if P[i] >= K:
        print(i+1)
        exit()

print(-1)