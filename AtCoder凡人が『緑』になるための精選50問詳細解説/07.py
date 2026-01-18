H,W,X,Y = map(int,input().split())
S = []
for i in range(H):
    S.append(list(input()))

i = 1
cnt = 1

while True:
    if Y-1-i >= 0 and S[X-1][Y-1-i] == ".":
        cnt += 1
        i += 1
    else:
        break

i = 1

while True:
    if Y-1+i < W and S[X-1][Y-1+i] == ".":
        cnt += 1
        i += 1
    else:
        break

i = 1

while True:
    if X-1-i >= 0 and S[X-1-i][Y-1] == ".":
        cnt += 1
        i += 1
    else:
        break

i = 1

while True:
    if X-1+i < H and S[X-1+i][Y-1] == ".":
        cnt += 1
        i += 1
    else:
        break

print(cnt)

# 17 鬼の条件分岐で解いてしまった。マス目系は苦手なので克服したい。