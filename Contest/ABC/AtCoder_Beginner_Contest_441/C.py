N,K,X = map(int,input().split())
A = list(map(int,input().split()))

sake_sort = sorted(A, reverse=True)

# 水の数
water = N-K
# 酒の最小範囲
sake = sake_sort[water:]

sake.sort

total = 0

for i in range(len(sake)):
    total += sake[i]
    if total >= X:
        print(i+1+water)
        exit()

print(-1)