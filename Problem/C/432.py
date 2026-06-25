N, X, Y = map(int,input().split())
A = list(map(int,input().split()))

# 最大と最小のグラムを計算して、Y-Xの範囲内で重複していたらできる

max_a = float("inf")
min_a = 0
calc = (A[0]*X) % (Y-X)

for a in A:
    max_a = min(max_a,a*Y)
    min_a = max(min_a,a*X)

    if calc != (a*X) % (Y-X):
        print(-1)
        exit()

# print(max_a,min_a)

if max_a < min_a:
    print(-1)
    exit()

diff_xy = Y-X
ans = 0

for a in A:
    diff = a*Y - max_a
    ans += a - diff//diff_xy

print(ans)