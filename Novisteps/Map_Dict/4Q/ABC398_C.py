from collections import defaultdict

N = int(input())
A = list(map(int,input().split()))

D = defaultdict(list)

for i in range(N):
    D[A[i]].append(i+1)

ans = -1
max_val = 0

for k,v in D.items():
    if len(v) == 1 and k > max_val:
        max_val = k
        ans = v[0]

print(ans)