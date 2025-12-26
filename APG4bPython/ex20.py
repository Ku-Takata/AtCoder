N = int(input())
A = list(map(int, input().split()))

cnt = {}

for a in A:
    if a in cnt:
        cnt[a] += 1
    else:
        cnt[a] = 1

max_cnt = 0
val = -1

for a in A:
    if max_cnt < cnt[a]:
        max_cnt = cnt[a]
        val = a

print(val, max_cnt)