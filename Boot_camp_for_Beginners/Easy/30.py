x1,y1,x2,y2 = map(int,input().split())

# 反時計回りに回転
vector = (x2-x1, y2-y1)
dx = vector[0]
dy = vector[1]

x3 = x2-dy
y3 = y2+dx
x4 = x3-dx
y4 = y3-dy

print(x3,y3,x4,y4)

# 35 ベクトルの回転の理解に時間がかかった。