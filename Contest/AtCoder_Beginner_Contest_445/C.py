N = int(input())
A = list(map(int,input().split()))
S = [i for i in range(1,N+1)]
"""
M = []

for i in range(N):
    move_list = [S[i]]
    for j in range(10**100):
        if A[move_list[-1]-1] not in move_list:
            move_list.append(A[move_list[-1]-1])
        else:
            M.append(move_list[-1])
            break

print(*M)
"""
# TLE1個、予想はしてたが以外といけてる。
# 制約を読み間違えていた。i < A[i]であることに気づくべきだった。
# この制約に気づけば、ループする場合が自分自身のマスの時だけと気づける。
# よくわからないままなんちゃって解法で解いていた。

N = int(input())
A = list(map(int, input().split()))

M = [0] * (N + 1)

for i in range(N, 0, -1):
    next_pos = A[i-1]

    if next_pos == i:
        M[i] = i
    else:
        M[i] = M[next_pos]

print(*M[1:])

# 問題文よく読みましょう案件