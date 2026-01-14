S = list(map(int, input()))

# 10! = 362880なので列挙可能
import itertools

comb = []
ans = float("inf")

# 数字の順番を変えても良い場合
"""
for i in range(len(S)+1):
    for num in itertools.permutations(S,i+1):
        comb.append(num)

for i in range(len(comb)):
    comb[i] = int("".join(map(str, comb[i])))
    if abs(comb[i] - 753) < ans:
        ans = abs(comb[i] - 753)
    if ans == 0:
        break
"""

# 今回は連続した部分文字列のみ、しかも3つだけ
for i in range(len(S)-2):
    if i < len(S)-3:
        sub = S[i:i+3]
    else:
        sub = S[i:]
    comb.append(sub)

for i in range(len(comb)):
    comb[i] = int("".join(map(str, comb[i])))
    if abs(comb[i]-753) < ans:
        ans = abs(comb[i]-753)

print(ans)

# 25 問題を間違えて読み取ってしまっていた。