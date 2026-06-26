N, K = map(int,input().split())
V = list(map(int,input().split()))

# K以上を満たす販売促進エリアの組の個数を探す
# セグ木でできるんだろうけど、できないから累積和

cs = [0]
for i in range(N):
    cs.append(cs[i]+V[i])

# print(cs)
ans = 0
L = 1

for i in range(1,N+1):
    now = cs[i]
    if now >= K:
        ans += L

    for j in range(L,N+1):
        if now - cs[j] >= K:
            ans += 1
            L += 1
        else:
            break
    # print(now,ans)

print(ans)