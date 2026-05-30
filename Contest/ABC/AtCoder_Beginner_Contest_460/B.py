T = int(input())

# つまり座標同士の最短距離を測って、それが半径2つを足し合わせた以上ならYes
# なぜ答えが一向に合わない？？？？
# 円とは円周らしい、意味わからん
# つまり中に円があったりするのはダメってことか

for i in range(T):
    X1, Y1, R1, X2, Y2, R2 = map(int,input().split())

    dist1 = (X2-X1)**2 + (Y2-Y1)**2
    dist2 = (R1+R2)**2
    dist3 = (R1-R2)**2
    # print(dist)

    if dist3 <= dist1 <= dist2:
        print("Yes")
    else:
        print("No")