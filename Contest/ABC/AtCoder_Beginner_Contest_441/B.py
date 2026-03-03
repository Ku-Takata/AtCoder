N, M = map(int,input().split())
S = input()
T = input()
Q = int(input())
W = []
for i in range(Q):
    W.append(list(input()))

ans1 = "Unknown"
ans2 = "Unknown"

for i in range(Q):
    for j in range(len(W[i])):
        # S側
        if W[i][j] not in S:
            ans1 = "Unknown"
            break
        else:
            ans1 = "Takahashi"
    for k in range(len(W[i])):
        # T側
        if W[i][k] not in T:
            ans2 = "Unknown"
            break
        else:
            ans2 = "Aoki"

    if ans2 == "Unknown" and ans1 == "Takahashi":
        print("Takahashi")
    elif ans1 == "Unknown" and ans2 == "Aoki":
        print("Aoki")
    else:
        print("Unknown")