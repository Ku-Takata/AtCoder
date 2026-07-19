N = int(input())
X = [list(map(int,input().split())) for i in range(N)]

# 上下左右を隣接するマスとする
# Xの隣接するマスは異なる色になっている
# Yの方は隣接するマスが同じでも良い
# Xの差が1なら、[1,3],[1,4],[2,4]とその逆順で合計6択
# Xの差が2以上なら、[1,1],[2,2],[3,3],[4,4],[1,2],[2,3],[3,4]とその逆順で合計14択
# 上下左右を毎回見ると、4*500*500 = 10**6
# 1つ決めると、全範囲に影響を及ぼす
# わ、わからん...

# Yのベースを全て1にする
Y = [[1]*N for i in range(N)]

for i in range(N):
    for j in range(N):
        for n,m in [[i-1,j],[i,j-1],[i+1,j],[i,j+1]]:
            if n < 0 or n > N-1 or m < 0 or m > N-1:
                continue

            if abs(X[i][j] - X[n][m]) >= 2:
                Y

print(Y)