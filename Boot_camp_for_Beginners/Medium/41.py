S = list(input())

# 1処理で実質的に2文字短縮可能？
# 問題理解した。
# これ一番頻度が高いかつ、その文字までの間の最小が最大の文字を起点に、
# 一回の処理で複数個をその文字に変えつつ、一文字減らせるということだ
# と思ったけど、起点の文字の条件間違っている気がするし、多分簡単に選べないから全探索でいい気がする。
# ある文字を選んで、それ以外の文字が最大どれくらい続くかを回答したら良い

import string
alphabet = list(string.ascii_lowercase)
max_length_list = []

for a in alphabet:
    if a not in S:
        continue
    length = 0
    max_length = 0
    for char in S:
        if a == char:
            max_length = max(max_length,length)
            length = 0
        else:
            length += 1

    max_length = max(max_length,length)
    max_length_list.append(max_length)

print(min(max_length_list))