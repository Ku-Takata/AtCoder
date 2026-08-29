N, K = map(int,input().split())
A = list(map(int,input().split()))

from collections import Counter

Class = Counter(A)
max_cnt = Class.most_common(1)[0][1]

# print(Class)
# print(max_cnt)
ans = 0

for v in Class.values():
    if max_cnt == v or max_cnt == v+1:
        ans += 1

print(ans)