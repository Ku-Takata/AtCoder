N = int(input())
A = list(map(int,input().split()))

from collections import Counter

cnt_A = Counter(A)
ans = 0

for v in cnt_A.values():
    ans += v*(v-1)*(N-v) // 2

print(ans)