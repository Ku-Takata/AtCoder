N, K = map(int,input().split())

sale = []
total = 0

# クーポン無し総和からセール値が大きいものを順番に引く
for i in range(N):
    A, B = map(int,input().split())
    total += A
    sale.append(A-B)

sale.sort(reverse=True)

for i in range(K):
    total -= sale[i]

print(total)