N, M = map(int,input().split())
C = list(map(str,input().split()))
D = list(map(str,input().split()))
P = list(map(int,input().split()))

price = dict()

for i in range(M):
    price[D[i]] = P[i+1]

ans = 0

for c in C:
    if c in D:
        ans += price[c]
    else:
        ans += P[0]

print(ans)