N = int(input())
S = []
for i in range(N):
    S.append(list(input()))

print(S)

# たぶん上からの行と下からの行が同じかつ、Nが奇数なら真ん中が挟まれている行を反転したものであればOK
"""
for i in range(N//2):
    for j in range(N):
        if S[i][j] != S[-i-1][j]:
            if S[i][j] == "?" or S[-i-1][-j-1] == "?":
                
            else:
                print(-1)
                exit()

if N % 2 == 1:
    for i in range(N):
        if S[N//2][i] != 

"""
# 周りの値が全て同じなら良い？
# ?の場合もOKとする

for i in range(N):
    for j in range(N):
        if i == 0:
            if j == 0:
                if S[i+1][j] != S[i][j+1]:
                    if S[i+1][j] == "?" and S[i][j+1] == "?":
                        S[i+1][j] = 0
                        S[i][j+1] = 0
                    elif S[i+1][j] == "?":
                        S[i+1][j] == S[i][j+1]
                    elif S[i][j+1] == "?":
                        S[i][j+1] == S[i+1][j]
                    else:
                        print(-1)
                        exit()
            elif j != N-1:
                if S[i+1][j] != S[i][j+1] != S[i][j-1]:
                    if S[i+1][j] == "?" and S[i][j+1] == "?" and S[i][j-1] == "?":
                        S[i+1][j] = 0
                        S[i][j+1] = 0
                        S[i][j-1] = 0
                    elif S[i+1][j] == "?" and S[i][j+1] == "?":
                        S[i+1][j] = S[i][j-1]
                        S[i][j+1] = S[i][j-1]
                    elif S[i][j+1] == "?" and S[i][j-1] == "?":
                        S[i][j+1] = S[i+1][j]
                        S[i][j-1] = S[i+1][j]
                    elif S[i+1][j] == "?" and S[i][j-1] == "?":
                        S[i+1][j] = S[i][j+1]
                        S[i][j-1] = S[i][j+1]
                    elif S[i+1][j] == "?":
                        S[i+1][j] == S[i][j+1]
                    elif S[i][j+1] == "?":
                        S[i][j+1] == S[i+1][j]
                    elif S[i][j-1] == "?":
                        S[i][j-1] = S[i][j+1]
                    else:
                        print(-1)
                        exit()
            else:
                if S[i+1][j] != S[i][j-1]:
                    if S[i+1][j] == "?" and S[i][j-1] == "?":
                        S[i+1][j] = 0
                        S[i][j-1] = 0
                    elif S[i+1][j] == "?":
                        S[i+1][j] == S[i][j-1]
                    elif S[i][j-1] == "?":
                        S[i][j-1] == S[i+1][j]
                    else:
                        print(-1)
                        exit()
        elif i == N-1:
            if j == 0:
                if S[i-1][j] != S[i][j+1]:
                    if S[i-1][j] == "?" and S[i][j+1] == "?":
                        S[i-1][j] = 0
                        S[i][j+1] = 0
                    elif S[i-1][j] == "?":
                        S[i-1][j] == S[i][j+1]
                    elif S[i][j+1] == "?":
                        S[i][j+1] == S[i-1][j]
                    else:
                        print(-1)
                        exit()
            elif j != N-1:
                if S[i-1][j] != S[i][j+1] != S[i][j-1]:
                    if S[i-1][j] == "?" and S[i][j+1] == "?" and S[i][j-1] == "?":
                        S[i-1][j] = 0
                        S[i][j+1] = 0
                        S[i][j-1] = 0
                    elif S[i-1][j] == "?" and S[i][j+1] == "?":
                        S[i-1][j] = S[i][j-1]
                        S[i][j+1] = S[i][j-1]
                    elif S[i][j+1] == "?" and S[i][j-1] == "?":
                        S[i][j+1] = S[i-1][j]
                        S[i][j-1] = S[i-1][j]
                    elif S[i-1][j] == "?" and S[i][j-1] == "?":
                        S[i-1][j] = S[i][j+1]
                        S[i][j-1] = S[i][j+1]
                    elif S[i-1][j] == "?":
                        S[i-1][j] == S[i][j+1]
                    elif S[i][j+1] == "?":
                        S[i][j+1] == S[i-1][j]
                    elif S[i][j-1] == "?":
                        S[i][j-1] = S[i][j+1]
                    else:
                        print(-1)
                        exit()
            else:
                if S[i-1][j] != S[i][j-1]:
                    if S[i-1][j] == "?" and S[i][j-1] == "?":
                        S[i-1][j] = 0
                        S[i][j-1] = 0
                    elif S[i-1][j] == "?":
                        S[i-1][j] == S[i][j-1]
                    elif S[i][j-1] == "?":
                        S[i][j-1] == S[i-1][j]
                    else:
                        print(-1)
                        exit()
        else:
            if j != 0 and j != N-1:
                if S[i+1][j] != S[i][j+1] != S[i][j-1] != S[i-1][j]:
                        if S[i+1][j] == "?" and S[i][j+1] == "?" and S[i][j-1] == "?" and S[i-1][j] == "?":
                            S[i+1][j] = 0
                            S[i][j+1] = 0
                            S[i][j-1] = 0
                            S[i-1][j] = 0
                        elif S[i+1][j] == "?" and S[i][j+1] == "?" and S[i][j-1] == "?":
                            S[i+1][j] = 0
                            S[i][j+1] = 0
                            S[i][j-1] = 0
                        elif S[i+1][j] == "?" and S[i][j+1] == "?" and S[i-1][j] == "?":
                            S[i+1][j] = 0
                            S[i][j+1] = 0
                            S[i-1][j] = 0
                        elif S[i+1][j] == "?" and S[i][j-1] == "?" and S[i-1][j] == "?":
                            S[i+1][j] = 0
                            S[i][j-1] = 0
                            S[i-1][j] = 0
                        elif S[i][j+1] == "?" and S[i][j-1] == "?" and S[i-1][j] == "?":
                            S[i][j+1] = 0
                            S[i][j-1] = 0
                            S[i-1][j] = 0
                        elif S[i+1][j] == "?" and S[i][j+1] == "?":
                            S[i+1][j] = S[i][j-1]
                            S[i][j+1] = S[i][j-1]
                        elif S[i][j+1] == "?" and S[i][j-1] == "?":
                            S[i][j+1] = S[i+1][j]
                            S[i][j-1] = S[i+1][j]
                        elif S[i+1][j] == "?" and S[i][j-1] == "?":
                            S[i+1][j] = S[i][j+1]
                            S[i][j-1] = S[i][j+1]
                        elif S[i+1][j] == "?" and S[i-1][j] == "?":
                            S[i+1][j] = S[i][j+1]
                            S[i-1][j] = S[i][j+1]
                        elif S[i][j+1] == "?" and S[i-1][j] == "?":
                            S[i][j+1] = S[i+1][j]
                            S[i-1][j] = S[i+1][j]
                        elif S[i][j-1] == "?" and S[i-1][j] == "?":
                            S[i][j-1] = S[i+1][j]
                            S[i-1][j] = S[i+1][j]
                        elif S[i+1][j] == "?":
                            S[i+1][j] == S[i][j+1]
                        elif S[i][j+1] == "?":
                            S[i][j+1] == S[i+1][j]
                        elif S[i][j-1] == "?":
                            S[i][j-1] = S[i][j+1]
                        elif S[i-1][j] == "?":
                            S[i-1][j] = S[i][j+1]
                        else:
                            print(-1)
                            exit()
            if j == 0:
                None
            if j == N-1:
                None

print(S)

# 順番に周りを同じにしていけば良いとは分かったけど、実装力なさすぎて気が狂ったように条件分岐を書いてしまった。
# たぶんヒューマンエラーも起こりまくってる。
# でも途中から意地になってパワー系をやってしまった。
# どうやら反対角で同じ数字になるようになれば良いらしい。そっちだったか～泣