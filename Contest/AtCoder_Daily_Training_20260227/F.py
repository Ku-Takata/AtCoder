N = int(input())
A = list(map(int,input().split()))

from collections import Counter

cnt_A = Counter(A)
ans = 0

for k,v in cnt_A.items():
    ans += v//2

print(ans)