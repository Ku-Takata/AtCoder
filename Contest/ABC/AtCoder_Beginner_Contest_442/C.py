N,M = map(int,input().split())
cnt_rigai = [0] * (N+1)

for i in range(M):
    A,B = map(int,input().split())
    cnt_rigai[A] += 1
    cnt_rigai[B] += 1

# AB以外の組み合わせを数える
"""import itertools

for i in range(1,N+1):
    ans = len(list(itertools.combinations(range())))"""

"""
from math import factorial

ans_list = []

for i in range(1,N+1):
    if N-cnt_rigai[i]-1 >= 3:
        ans = factorial(N-cnt_rigai[i]-1) / (factorial(3) * factorial(N-cnt_rigai[i]-1-3))
        ans_list.append(int(ans))
    else:
        ans_list.append(0)

print(*ans_list)
"""

from scipy.special import comb

ans_list = []

for i in range(1,N+1):
    if N-cnt_rigai[i]-1 >= 3:
        ans = comb((N-cnt_rigai[i]-1), 3, exact=True)
    else:
        ans = 0
    
    ans_list.append(ans)

print(*ans_list)

# 組み合わせのライブラリとして初めてscipyを使った。いつもは組み合わせの列挙が多いが、今回は列挙の必要がなかった。
# そこで少し手こずり、1ペナ40分弱かかってしまった。