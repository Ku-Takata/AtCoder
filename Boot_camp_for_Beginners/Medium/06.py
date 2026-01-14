H,W = map(int,input().split())
S = []

for i in range(H):
    S.append(list(input()))

for i in range(H):
    for j in range(W):
        cnt = 0
        if S[i][j] == ".":
            if i > 0:
                if S[i-1][j] == "#":
                    cnt += 1
                if j < W-1:
                    if S[i-1][j+1] == "#":
                        cnt += 1
                if j > 0:
                    if S[i-1][j-1] == "#":
                        cnt += 1
            if i < H-1:
                if S[i+1][j] == "#":
                    cnt += 1
                if j < W-1:
                    if S[i+1][j+1] == "#":
                        cnt += 1
                if j > 0:
                    if S[i+1][j-1] == "#":
                        cnt += 1
            if j < W-1:
                if S[i][j+1] == "#":
                    cnt += 1
            if j > 0:
                if S[i][j-1] == "#":
                    cnt += 1

            S[i][j] = cnt

for i in range(H):
    print("".join(map(str,S[i])))

# 20 場合分けが面倒だった...スマートな解法が書けるようになる必要がある。