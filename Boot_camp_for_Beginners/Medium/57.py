from collections import Counter

N = int(input())
S = ["".join(sorted(input())) for i in range(N)]

cnt = Counter(S)
total = 0

for v in cnt.values():
    total += v*(v-1) // 2

print(total)