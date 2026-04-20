X = int(input())

# 商品購入可能回数(最大)
N,mod = divmod(X,100)

if 0 <= mod <= N*5:
    print(1)
else:
    print(0)