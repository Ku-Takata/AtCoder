N, M = map(int,input().split())
C = list(map(int,input().split()))

from collections import Counter

total = 0

for i in range(N):
    K = int(input())
    P = list(map(int,input().split()))

    cnt_P = Counter(P)

    for k,v in cnt_P.items():
        if v <= C[k-1]:
            total += v

print(total)