N, Q = map(int,input().split())
S = [input() for i in range(N)]
ab = [list(map(str,input().split())) for i in range(Q)]

import string
alphabet = list(string.ascii_lowercase)

# 特定の英小文字が最終的に何になるのかを辞書にする
for i in range(Q):
    for j in range(26):
        if alphabet[j] == ab[i][0]:
            alphabet[j] = ab[i][1]
alpha_dict = {}
for i,c in enumerate(range(ord('a'),ord('z')+1)):
    alpha_dict[chr(c)] = alphabet[i]

for i in range(N):
    changed_S = "".join([alpha_dict[char] for char in S[i]])
    print(changed_S)

# 全ての文字列の総和は10**6以下であるから行けた。
# 最初1つの文字列につき10**5個以下で総和に制限は無いと思っていたからTLEになると思っていた。