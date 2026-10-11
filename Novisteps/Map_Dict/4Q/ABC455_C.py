from collections import Counter

N, K = map(int,input().split())
A = list(map(int,input().split()))

cnt_A = Counter(A)
val = list()

for k,v in cnt_A.items():
    val.append(k*v)

val.sort(reverse=True)
# print(val)

ans = 0

for i in range(len(val)):
    if i >= K:
        ans += val[i]

print(ans)