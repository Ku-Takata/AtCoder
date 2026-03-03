S = list(input())

from collections import Counter

cnt_S = Counter(S)
max_cnt = max(cnt_S.values())

for k,v in cnt_S.items():
    if v == max_cnt:
        for _ in range(v):
            S.remove(k)

print("".join(S))