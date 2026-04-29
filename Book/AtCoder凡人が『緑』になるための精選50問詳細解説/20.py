N = int(input())
S = list(input())
Q = int(input())

# switchが0で通常状態
switch = 0

for i in range(Q):
    T, A, B = map(int,input().split())
    A -= 1
    B -= 1
    if T == 1:
        if switch == 1:
            A += N
            A %= (2*N)
            B += N
            B %= (2*N)
        S[A], S[B] = S[B], S[A]
    else:
        if switch == 0:
            switch = 1
        else:
            switch = 0

if switch == 1:
    S = S[N:] + S[:N]

print("".join(S))

# 45 解法も実装もあまりピンと来ず、他の人のコードを参考に理解した。文字列操作系は結構苦手なので、演習が必要。