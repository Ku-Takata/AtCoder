H, W, K = map(int,input().split())
c1, c2 = map(str,input().split())
S = [list(input()) for i in range(H)]
ans = []

for i in range(H):
    temp = []
    for j in range(W):
        if S[i][j] == "#":
            temp.append(c1*K)
        else:
            temp.append(c2*K)
    for j in range(K):
        ans.append(temp)

for i in range(len(ans)):
    print("".join(ans[i]))