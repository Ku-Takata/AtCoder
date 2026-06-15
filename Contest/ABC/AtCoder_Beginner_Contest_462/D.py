N, D = map(int,input().split())
ST = []

# 区間和的なやつか？
# とりあえず累積っぽくしてみる

# まずD時間未満しかいないものは省く
for i in range(N):
    S,T = map(int,input().split())

    if T-S < D:
        continue
    else:
        ST.append([S,T])

diff = [0]*(10**6 + 10)

for i in range(len(ST)):
    diff[ST[i][0]] += 1
    diff[ST[i][1]-D+1] -= 1

ans = 0
cnt = 0

for i in range(1,10**6 + 10):
    cnt += diff[i]
    ans += cnt * (cnt-1) // 2

print(ans)