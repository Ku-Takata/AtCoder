N, K = map(int,input().split())
AB = [list(map(int,input().split())) for i in range(N)]

ans = 0

for i in range(1,101):
    for j in range(1,101):
        kagen = [i,j]
        cnt = sum(1 for x in AB if kagen[0] <= x[0] <= kagen[0]+K and kagen[1] <= x[1] <= kagen[1]+K)
        if cnt > ans:
            ans = cnt

print(ans)