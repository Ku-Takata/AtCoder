H, W = map(int,input().split())
X = [list(map(int,input().split())) for i in range(H)]
Q = int(input())

cs_horizon = []
cs = [[0]*(W+1)]

# まず横方向に累積和
for i in range(H):
    temp = []
    for j in range(W):
        if j == 0:
            temp.append(X[i][j])
        else:
            temp.append(temp[-1]+X[i][j])
    cs_horizon.append(temp)

# print(cs_horizon)

# 横方向の累積和を縦方向に累積和
for i in range(H):
    temp = [0]
    for j in range(W):
        temp.append(cs_horizon[i][j]+cs[i][j+1])
    cs.append(temp)

# print(cs)

for i in range(Q):
    A, B, C, D = map(int,input().split())
    ans = cs[A-1][B-1] + cs[C][D] - cs[A-1][D] - cs[C][B-1]
    print(ans)