T = int(input())
for i in range(T):
    N = int(input())
    WP = [list(map(int,input().split())) for i in range(N)]
    WP.sort(key=lambda x: x[0] + x[1])
    Power = 0
    total = 0

    for w,p in WP:
        Power += p

    for i in range(N):
        total += sum(WP[i])
        if total > Power:
            print(i)
            break

# 数学的な要素が強めな問題だった。貪欲法。