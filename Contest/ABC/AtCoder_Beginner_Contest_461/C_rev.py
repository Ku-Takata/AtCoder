N, K, M = map(int,input().split())
CV = [list(map(int,input().split())) for i in range(N)]

# M種類選ぶのは確定で、K個選ぶ
# だからまずは各色の中で最も大きい宝石をリランキングし、大きいものから取る
# 取る際はK個までに注意
# その後、取ったもの以外で大きい宝石をK個まで取る
# 後々のためにCVを辞書にしたい

