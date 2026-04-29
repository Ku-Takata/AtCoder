N,X = map(int,input().split())
sake = []

for i in range(N):
    sake.append(list(map(int,input().split())))

# 少数計算は誤差が出ることを考慮
alcohol = 0

for i in range(N):
    alcohol += sake[i][0] * sake[i][1]
    if alcohol > X * 100:
        print(i+1)
        exit()

print(-1)

# 8 整数で計算することに注意することを学んだ。