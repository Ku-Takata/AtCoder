N,X = map(int,input().split())
A = list(map(int,input().split()))

A_rev = []

for i in range(N):
    if A[i] != X:
        A_rev.append(A[i])

print(*A_rev)

# 8 最初はXの出現回数をカウントしてカウント分だけ削除しようとしたが、よく考えたらそれだと探索をカウント*NになるのでTLEになることが分かった。