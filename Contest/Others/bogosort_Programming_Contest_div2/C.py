T = int(input())

# 3文字回文になっている所を探す
# そこから3文字回文同士でくっつけられる

for i in range(T):
    S = input()
    K = int(input())

    idx = []
    ans = 0

    for i in range(len(S)-2):
        if S[i] == S[i+2]:
            idx.append((i,i+2))
            ans += 1

    # print(idx)

    for i in range(len(idx)-1):
        for j in range(i+1,len(idx)):
            if idx[j][1] - idx[i][0] + 1 <= K:
                ans += 1
            else:
                break

    print(ans)