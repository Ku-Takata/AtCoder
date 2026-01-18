N, K = map(int,input().split())

for i in range(K):
    N = list(str(N))
    g1 = sorted(N, reverse=True)
    g2 = sorted(N)
    g1 = int("".join(g1))
    g2 = int("".join(g2))
    f = g1 - g2
    N = f

print(N)

# 27 数値をリストに変換するために文字列に直すこと、そしてリスト化されたものをスペースなしでくっつける方法joinを学んだ。