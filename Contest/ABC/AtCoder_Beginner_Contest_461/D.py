H, W, K = map(int,input().split())
S = [(int(input())) for i in range(H)]

# これどうやって探索するの？？
# 1空間ずつ拡張して全探索しか思いつかん
# 空間の種類の計算が分からん
# H(H+1)/2 * W(W+1)/2 らしい

space_comb = []

for i in range(H):
    for j in range(W):
        space_comb.append()