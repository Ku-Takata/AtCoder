N, K = map(int,input().split())
S = list(map(int, str(input())))

from collections import Counter

cnt_S = Counter(S)
if 0 not in cnt_S.keys():
    cnt_S[0] = 0
if 1 not in cnt_S.keys():
    cnt_S[1] = 0

mod = K % 2

if mod == cnt_S.get(1) % 2:
    print("Yes")
else:
    print("No")