N = int(input())
S = []
T = []

for i in range(N):
    s,t = input().split()
    S.append(s)
    T.append(int(t))

import heapq

second_highest = heapq.nlargest(2,T)[1]

print(S[T.index(second_highest)])

# 20 文字列と数字が同時に入力されたときのリスト分け方法を学んだ。あと何番目に大きい小さいを取得する方法を学んだ。
# 以下は著者のコードを理解して、何も見ずに書いたもの。こっちの方が簡単かつ値が同じときに文字列の辞書順にも対応できるため、こっちの方が良い。

"""
N = int(input())
mountains = []

for i in range(N):
    S,T = map(str,input().split())
    T = int(T)
    mountains.append([T,S])

mountains.sort(reverse=True)

print(mountains[1][1])
"""