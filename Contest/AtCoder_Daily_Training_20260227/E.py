N = int(input())
S = list(input())
Q = int(input())

C = []
D = []

# 更新するやり方を最終的に更新するかを書き換えたい
for i in range(Q):
    c, d = map(str,input().split())
    if c in D:
        D[D.index(c)] = d
    elif c not in C:
        C.append(c)
        D.append(d)

print(C,D)
for i in range(N):
    for j in range(len(C)):
        if S[i] == C[j]:
            S[i] = D[j]

print(S)

# WA、上手くいかない方法