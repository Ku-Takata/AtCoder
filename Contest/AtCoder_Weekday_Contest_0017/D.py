N, M, K = map(int,input().split())
A = list(map(int,input().split()))
B = [list(map(int,input().split())) for i in range(M)]

from itertools import combinations

comb = list(combinations(A,K))
comb_i = list(combinations(enumerate(A),K))
ans = float("-inf")

for i in range(len(comb)):
    select = {index[0] for index in comb_i[i]}
    total = sum(comb[i])
    for j in range(M):
        if B[j][0]-1 in select and B[j][1]-1 in select:
            total -= B[j][2]

    if total > ans:
        ans = total

print(ans)