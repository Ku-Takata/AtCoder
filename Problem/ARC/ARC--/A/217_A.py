import itertools

T = int(input())

# 2ビット演算、XORの結果が最小になるような順列を出力
# この計算は順列をたくさん並べても0に収束する
# だからN自体はそこまで大きくできない
# つまり最初に各Nの値毎のリストを全列挙したら解けそう
# Nの最大値は9
# 1~Nの各要素を並び替える
# 一応人力で解けそうではある
# 

N_table = dict()

for i in range(1,4):
    N_comb = []
    N_list = list(itertools.permutations(range(1,i+1),i))
    N_comb.append(N_list)

    # 各nc毎にXOR総和を取る
    for nc in N_comb:
        print(nc)

for i in range(T):
    N = int(input())

