N = int(input())

# その競技にいくつチームがあるのかをリスト化したい
from collections import defaultdict
PQ = defaultdict(lambda: defaultdict(int))

for i in range(N):
    p, q = map(int,input().split())
    PQ[p][q] += 1

# print(PQ)
import math

ans = 0

for p,q in PQ.items():
    player = sum(q.values())
    total = math.comb(player,2)

    same_team = 0
    for k,v in q.items():
        same_team += math.comb(v,2)

    ans += total - same_team

print(ans)