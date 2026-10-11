from collections import defaultdict

N = int(input())
A = list(map(int,input().split()))

A_idx = defaultdict(list)

for i in range(N):
    A_idx[A[i]].append(i+1)

# print(A_idx)

ans = list()
cnt = defaultdict(int)

for i in range(N):
    cnt[A[i]] += 1

    if cnt[A[i]] == 1:
        ans.append(-1)
    else:
        ans.append(A_idx[A[i]][cnt[A[i]]-2])

print(*ans)