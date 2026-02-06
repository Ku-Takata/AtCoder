N = int(input())
A = []

for i in range(N):
    A.append(int(input()))

from collections import Counter

cnt_A = Counter(A)
ans = 0

for k,v in cnt_A.items():
    if v % 2 == 1:
        ans += 1

print(ans)

# 全探索しかけたが、すぐにTLEになると気付き、Counter使えば解けると気付けた。