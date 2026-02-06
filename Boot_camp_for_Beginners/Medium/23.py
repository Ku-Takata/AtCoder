N,M = map(int,input().split())

num = ["unk"]*N

for i in range(M):
    s,c = map(int,input().split())
    if num[s-1] == "unk":
        num[s-1] = c
    elif num[s-1] != c:
        print(-1)
        exit()

if num[0] == 0 and N != 1:
    print(-1)
    exit()

if num[0] == "unk" and N != 1:
    num[0] = 1

for i in range(N):
    if num[i] == "unk":
        num[i] = 0

print("".join(map(str,num)))

# 問題の読み取り間違いと実装に時間がかかってしまった。