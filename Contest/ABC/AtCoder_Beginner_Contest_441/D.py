N,M,L,S,T = map(int,input().split())
UVC = []

for i in range(M):
    UVC.append(list(map(int,input().split())))

# グラフは初見
# まずちょうどL回で移動できる頂点を探す？でも全列挙で10^6ちょいあるからダメそう
pos = 1
cost = 0

for i in range(M):
    for j in range(L):
        None #ここまでしか書いていない



# 初見過ぎて何も分からんかった...
# DFSで解けるらしい。理論的には既知だが、まだ実装した事がないのでやってみる必要がある。