N = int(input())

N_list = []

for i in range(1,N+1):
    if '7' not in list(str(i)) and '7' not in list(str(oct(i))):
        N_list.append(i)

# print(N_list)
print(len(N_list))

# 10 別にリストに格納する必要はないが、どうなっているか確認するためにリストを使った。
# もっと早いプログラムを書くなら、カウントしてそれを表示させるだけ。
# また今回は組み込み関数を使って8進数かどうか判定したが、以下でもできる。

"""
oct_digit = []
while ten_sinsuu > 0:
    mod = ten_sinsuu % 8
    oct_digit.append(str(mod))
    ten_sinsuu //= 8

oct_disit.sort(reverse=True)
"""