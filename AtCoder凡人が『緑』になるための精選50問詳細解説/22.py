N,K = map(int,input().split())
P = list(map(int,input().split()))

# windowがKで区間和を求め、最大の期待値を求める。
# 先に期待値に直す
for i in range(N):
    P[i] = P[i] / 2 + 0.5

# 区間和は累積和を先に求める
cs = []
total = 0

for i in range(N):
    total += P[i]
    cs.append(total)

ans = 0

for i in range(K-1,N):
    if i != K-1:
        if cs[i] - cs[i-K] > ans:
            ans = cs[i] - cs[i-K]
    else:
        ans = cs[i]

print(ans)

# 25 区間和の時、愚直にやる癖がある。「区間和は累積和を求めてから」を癖付けたい。