S = list(input())

# 1文字は確定で消さないといけない
S.pop(-1)

# 半分までの文字列と同じ文字列であれば最大
for i in range(len(S)):
    if len(S) % 2 != 0:
        S.pop(-1)
        continue
    # print(S[0:len(S)//2], S[len(S)//2:])
    if S[0:len(S)//2] == S[len(S)//2:]:
        break
    else:
        S.pop(-1)

print(len(S))

# 14