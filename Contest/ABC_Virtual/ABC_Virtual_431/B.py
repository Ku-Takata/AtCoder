X = int(input())
N = int(input())
W = list(map(int,input().split()))
Q = int(input())
P = [int(input())-1 for i in range(Q)]

parts = [False] * N
ans = X

for i in range(Q):
    if not parts[P[i]]:
        parts[P[i]] = True
        ans += W[P[i]]
    else:
        parts[P[i]] = False
        ans -= W[P[i]]

    print(ans)