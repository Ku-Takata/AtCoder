H, W = map(int,input().split())
S = [input() for i in range(H)]

# 点対称分からない問題発生
# とりあえず制約小さいし全探索で
# なんか形式的には～のとこが答えなんじゃない？

import itertools

ans = 0

for h1,h2,w1,w2 in itertools.product(range(H),range(H),range(W),range(W)):
    if h1 <= h2 and w1 <= w2:
        if all(S[i][j] == S[h1+h2-i][w1+w2-j]
            for i in range(h1,h2+1)
            for j in range(w1,w2+1)):
            ans += 1

print(ans)