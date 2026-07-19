T = int(input())

# 円周上というか実際には整数しかないから中心とする正四角形と言えるな
# まず中心点を探す方が簡単そう？
# 中心点は点と点の間のパターンと、それぞれの点のX軸とY軸同士が交差する点パターンがあって全部で3種類
# 円周上にあるかどうかって中心点であれば、全部円周上にあるくね？
# ↑ 普通に違った
# 中心点って3種じゃないなこれ、2つの点の線分の垂直2等分線上にある点だから無数にある
# だからその垂直2等分線が交わるのであれば、共通する中心点があるってこと
# つまりそれぞれの2点を結ぶ線が平行でなければ中心点が存在しうる
# あと平行でも同じ線上にあるなら交わっているから中心点がある

"""
for _ in range(T):
    Px, Py, Qx, Qy, Rx, Ry, Sx, Sy = map(int,input().split())

    # 中心点
    center1 = ((Px+Qx)/2,(Py+Qy)/2)
    center2 = (Px,Qy)
    center3 = (Py,Qx)

    center4 = ((Rx+Sx)/2,(Ry+Sy)/2)
    center5 = (Rx,Sy)
    center6 = (Ry,Sx)

    center_PQ = [center1,center2,center3]
    center_RS = [center4,center5,center6]

    C = list()
    for i in range(3):
        for j in range(3):
            if center_PQ[i] == center_RS[j]:
                C.append(center_PQ[i])

    if len(C) == 0:
        print("No")
        continue

    print("Yes")
"""

# 平行かどうかと同じ線分上の点か見る
for _ in range(T):
    Px, Py, Qx, Qy, Rx, Ry, Sx, Sy = map(int,input().split())

    dx1,dy1 = Qx-Px, Qy-Py
    dx2,dy2 = Sx-Rx, Sy-Ry

    if dx1*dy2 == dy1*dx2:
        temp = ((Px+Qx)-(Rx+Sx))*dx2 + ((Py+Qy)-(Ry+Sy))*dy2
        if temp == 0:
            print("Yes")
        else:
            print("No")
    else:
        print("Yes")