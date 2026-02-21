N = int(input())
XY = [list(map(int,input().split())) for i in range(N)]
Q = int(input())
ABCD = [list(map(int,input().split())) for i in range(Q)]

matrix = [[0]*1501 for i in range(1501)]

for i in range(N):
    matrix[XY[i][1]][XY[i][0]] += 1

# 横方向に累積和
cs = [[0]*1501 for i in range(1501)]
for i in range(1, 1501):
    for j in range(1, 1501):
        cs[i][j] = cs[i][j-1] + matrix[i][j]

# 縦方向に累積和
cs2 = [[0]*1501 for i in range(1501)]
for i in range(1, 1501):
    for j in range(1, 1501):
        cs2[i][j] = cs2[i-1][j] + cs[i][j]

for i in range(Q):
    a, b, c, d = ABCD[i]
    ans = cs2[d][c] - cs2[b-1][c] - cs2[d][a-1] + cs2[b-1][a-1]
    print(ans)