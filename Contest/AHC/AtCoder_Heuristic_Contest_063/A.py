# 体の色を揃えるのと、処理回数が少ない方が良い
# 胴体は座標が被ると千切れる
# 尻尾は座標の被りを許す
# 最大ターン数は10**5
# 移動先はUターン不可で、それ以外の上下左右
# 餌を食べると尻尾の位置に餌の色の尻尾が追加され、元の尻尾は胴体になる
# 胴体だけ嚙みちぎれる
# 噛みちぎられた胴体はその時点での座標に同じ色の餌を落とす

# 盤面サイズが8以上16以下、色数Cが3以上7以下、と制約はかなり小さい

import sys
input = sys.stdin.readline

N, M, C = map(int,input().split())
color = list(map(int,input().split()))
mass = [list(map(int,input().split())) for i in range(N)]
