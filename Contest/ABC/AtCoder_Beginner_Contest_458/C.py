S = input()

# Cの数を数えてそれを中心に左と右にいくつ拡張できるかをインデックスから確認する

ans = 0

for i in range(len(S)):
    if S[i] == "C":
        L,R = i,len(S)-i-1
        ans += min(L,R) + 1

print(ans)