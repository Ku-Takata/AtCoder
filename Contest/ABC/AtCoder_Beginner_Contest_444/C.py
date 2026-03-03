N = int(input())
A = list(map(int,input().split()))

# ある2つを足し合わせてそれが全体で統一できればLである
"""
from itertools import combinations

comb1 = list(set(combinations(A,2)))
comb2 = list(set(combinations(A,1)))
comb = comb1 + comb2
cnt = 0

print(comb)
"""
# 中央値か！？
import statistics

mid = statistics.median(A)


# 2個あるパターンを考える
if A.count(min(A)) % 2 == 0 and min(A)*2 == max(A):
    if N % 2 == 0:
        if min(A)*2 != int(mid*2):
            print(min(A)*2, int(mid*2))
        else:
            print(min(A)*2)
    else:
        if min(A)*2 != int(mid):
            print(min(A)*2, mid)
        else:
            print(min(A)*2)
else:
    if N % 2 == 0:
        print(int(mid*2))
    else:
        print(mid)