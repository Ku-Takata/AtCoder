T = int(input())

# 普通にDP?
# いや何回でも商品買ってもいいのか
# クーポンの数が残り商品数を上回ったらクーポンが余る事になり、無駄買いになる
# なので、クーポン数は最終的に使い切るようにしたい
# B-Aが大きいものを優先的にクーポンを使って購入したい
# そしてクーポンの稼ぎ先はB-Aが小さいもの

for i in range(T):
    N = int(input())
    AB = list(list(map(int,input().split())) for i in range(N))

    ans = []
    save = []
    base_cost = 0
    min_A = float("inf")

    for A,B in AB:
        base_cost += A
        save.append(B-A)
        min_A = min(A,min_A)

    save.sort()
    total_save = 0
    best_save = 0
    # print(save)

    for j in range(1,N+1):
        total_save += -save[j-1]
        add = max(0,2*j - N)
        calc = total_save - add*min_A
        best_save = max(best_save, calc)

    print(base_cost - best_save)
