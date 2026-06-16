N, A, B, C, D = map(int,input().split())
S = list(input())

# 普通にDPかなー、途中で交差する必要がある問題があるからそれに注意
# と思ったけど別にDP使わずとも解けそう
# 途中で入れ替わる必要がある場合は.が3つ以上続けば可、それ以外は不可
# あとはCとDの大小を見て可能かどうかを見れば良い
# #がゴール前に2個続いたら不可

A,B,C,D = A-1,B-1,C-1,D-1

for i in range(A,C-1):
    if S[i] == S[i+1] == "#":
        print("No")
        exit()
for i in range(B,D-1):
    if S[i] == S[i+1] == "#":
        print("No")
        exit()

# Bを先にゴールに行かせてからAをゴールさせる
if C < D:
    print("Yes")
else:
    change = False
    for i in range(B-1,min(D,N-2)):
        if S[i] == S[i+1] == S[i+2] == ".":
            change = True
            break

    if change:
        print("Yes")
    else:
        print("No")