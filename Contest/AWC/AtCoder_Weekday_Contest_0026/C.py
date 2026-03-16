N, T, E = map(int,input().split())
P = list(map(int,input().split()))

P = sorted(P)
cnt = 0
total = 0

for i in range(N):
    if total + T*P[i] <= E:
        cnt += 1
        total += T*P[i]

print(cnt)