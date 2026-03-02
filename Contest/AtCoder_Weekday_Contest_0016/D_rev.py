N, K, Q = map(int,input().split())
A = list(map(int,input().split()))

# 累積和
cs_A = [0]
for i in range(N):
    cs_A.append(cs_A[-1]+A[i])

# しゃくとりで重量を越える地点を0~Nまでのスタート地点それぞれで見つける
j = 0
over_i = []

for i in range(0,N):
    while j < N:
        if cs_A[j] - cs_A[i] <= K:
            j += 1
        else:
            break
    over_i.append(j)

# over_i (各倉庫から始めた時の停止位置 f(i)) の累積和を作る
cs_f = [0]
for i in over_i:
    cs_f.append(cs_f[-1] + i)

for i in range(Q):
    L, R = map(int, input().split())
    ans = cs_f[R] - cs_f[L-1]
    print(ans)