N = int(input())
S = [input() for i in range(N)]
M = int(input())
T = [input() for i in range(M)]

# Sにある文字列の数-Tにある文字列の数を最大化したい
# 制約的に愚直でいいのでは？

S_kinds = list(set(S))
ans = 0

for s in S_kinds:
    ans = max(ans,S.count(s)-T.count(s))

print(ans)