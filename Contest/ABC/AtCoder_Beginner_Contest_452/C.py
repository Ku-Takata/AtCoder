N = int(input())
AB = [list(map(int,input().split())) for i in range(N)]
M = int(input())
S = [list(input()) for i in range(M)]

# Nは小さい、Mは2*10**5と大きめ、文字列の長さは10までと小さめ
# Sは全て異なる文字列、全部小文字
# 脊椎に各文字列を1個ずつ決めて、条件が合うかどうかを判定する
# 最初に1文字目にaの文字列があるかどうか、2文字目にaの文字列があるかどうかといった感じで調べる
# 探索方法は一回のループでやりたいので、Sから1つずつ文字列を取り出し、何文字目にaみたいな感じで辞書に登録していくので良さそう

import string
alphabet = list(string.ascii_lowercase)
alphabet_dict = [dict() for i in range(10)]

for i in range(10):
    for char in alphabet:
        alphabet_dict[i][char] = 0
# print(alphabet_dict)

# 辞書登録
for i in range(M):
    for pos in range(len(S[i])):
        alphabet_dict[pos][S[i][pos]] = 1

# 解く
for i in range(M):
    backbone = S[i]
    if len(backbone) != N:
        print("No")
        continue

    for j in range(N):
        if backbone[AB]

# 気づいたけど文字列の長さも指定されているからこれだとダメだ
# 文字列の長さごとでアルファベット辞書を作れということ？