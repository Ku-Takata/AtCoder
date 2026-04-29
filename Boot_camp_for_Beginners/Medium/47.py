N, P = map(int,input().split())
A = list(map(int,input().split()))

# とりあえず奇数か偶数かのリストに直そう

for i in range(N):
    if A[i] % 2 == 0:
        A[i] = 0
    else:
        A[i] = 1

cnt_even = A.count(0)
cnt_odd = A.count(1)

from math import comb

ans = 0

if P == 0:
    for i in range(0,cnt_odd+1,2):
        ans += comb(cnt_odd,i)*(2**cnt_even)
    print(ans)
else:
    if cnt_odd <= 0:
        print(0)
    else:
        for i in range(1,cnt_odd+1,2):
            ans += comb(cnt_odd,i)*(2**cnt_even)
        print(ans)

# めちゃくちゃ複雑にしてしまったから簡単に書き直し。