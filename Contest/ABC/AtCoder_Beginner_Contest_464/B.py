H, W = map(int,input().split())
C = [list(input()) for i in range(H)]

# 横消し
flag = True
while flag:
    for j in range(W):
        if C[0][j] == "#":
            flag = False

    if not flag:
        break
    else:
        C.pop(0)
        H -= 1

flag = True
while flag:
    for j in range(W):
        if C[-1][j] == "#":
            flag = False

    if not flag:
        break
    else:
        C.pop(-1)
        H -= 1

flag = True
while flag:
    for j in range(H):
        if C[j][0] == "#":
            flag = False

    if not flag:
        break
    else:
        for i in range(H):
            C[i].pop(0)

flag = True
while flag:
    for j in range(H):
        if C[j][-1] == "#":
            flag = False

    if not flag:
        break
    else:
        for i in range(H):
            C[i].pop(-1)

for i in range(H):
    print("".join(C[i]))