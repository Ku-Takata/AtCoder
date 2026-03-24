N, K = map(int,input().split())
A = list(map(int,input().split()))

# リストの余りをソートして、隣合う要素の差が最大であるものを探す
# Kからそれを差し引いた値が最小値だと思われる
# 最初と最後の差も考慮する
# つまり円環から最大の隙間を引けば答え

mod_A = sorted([a % K for a in A])
max_sa = 0

for i in range(N-1):
    if mod_A[i+1] - mod_A[i] > max_sa:
        max_sa = mod_A[i+1] - mod_A[i]

edge_sa = mod_A[0]+K - mod_A[-1]
if edge_sa > max_sa:
    max_sa = edge_sa

print(K-max_sa)