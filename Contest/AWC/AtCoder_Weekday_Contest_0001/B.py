N, L, R = map(int,input().split())
P = list(map(int,input().split()))

point = -1
ans = -1

for i in range(N):
    if L <= P[i] <= R and P[i] > point:
        point = P[i]
        ans = i+1

print(ans)