N = int(input())
point = []

for i in range(N):
    point.append(list(map(int,input().split())))

# 3点が同じ線上に存在しているかどうかは、(y3-y1)(x2-x1) = (y2-y1)(x3-x1)が成り立てば存在している。
# 3重ループでも(10^2)^3=10^6でギリ計算可能
for i in range(N-2):
    for j in range(i+1, N-1):
        for k in range(j+1, N):
            if (point[k][1]-point[i][1])*(point[j][0]-point[i][0]) == (point[j][1]-point[i][1])*(point[k][0]-point[i][0]):
                print("Yes")
                exit()

print("No")

# 28 同じ線上に点がある式を学んだ。
# また、今回は3重ループで解いたが、N^2で解く方法がある。その方法であれば、3点以上でも対応可能。
# その手法はある1点から見て、残りの点の傾きを調べ、同じ傾きの数を数えることで解ける。