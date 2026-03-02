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

print(cs_A)
print(over_i)

for i in range(Q):
    L, R = map(int,input().split())
    if over_i[L] > R:
        ans_i = R
    else:
        ans_i = over_i[L]

    ans = cs_A[ans_i] - cs_A[L]
    print(ans)

# 重量がKを越えたらそこまでの重量を答えるのではなかった。
# スタート地点をL~Rの範囲で変えて、それぞれでの止まった所までの累積重量の総和を答える問題だった。