N,D = map(int,input().split())
X = []
for i in range(N):
    X.append(list(map(int,input().split())))

cnt = 0

for i in range(N):
    for j in range(i+1,N):
        total = 0
        for k in range(D):
            total += abs(X[i][k] - X[j][k])**2
        total **= 1/2

        if total.is_integer():
            cnt += 1

print(cnt)

# 実装面で混乱しかけた。