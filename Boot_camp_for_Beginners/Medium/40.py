N, M = map(int,input().split())
X = list(map(int,input().split()))

# 全然解法分からんな、とりあえずソートして距離の差配列に直すか
# たぶん差の配列を見て大きいやつをN-1個選んで、残りを足し合わせたのが答え？
# なんかなんとなくでしか分からんだけどAC

X.sort()
dif_len = [abs(X[i]-X[i+1]) for i in range(M-1)]
# print(X)
# print(dif_len)

dif_len.sort(reverse=True)
print(sum(dif_len[N-1:]))