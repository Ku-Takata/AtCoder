N = int(input())
A = list(map(int,input().split()))

from collections import Counter

cnt_A = Counter(A)
all_perm = 0

for k,v in cnt_A.items():
    all_perm += v*(v-1)//2

for i in range(N):
    cnt = cnt_A.get(A[i])
    perm_sa = (cnt*(cnt-1)//2) - ((cnt-1)*(cnt-2)//2)
    print(all_perm - perm_sa)

# 辞書参照はO(1)ではなくO(N)だと勘違いしており、これだとTLEになると予想したが普通にACで高速