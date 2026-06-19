S = list(str(input()))
K = int(input())

# 5000兆！欲しいな～これは
# それはさておき値がでかすぎる
# 1だけで構成される文字列であれば、答えは1だけど、それ以外は初めて現れた2以上の数字が答え

if len(set(S)) != 1:
    for i in range(len(S)):
        if S[i] != "1" and i+1 <= K:
            print(S[i])
            exit()
    print(1)
else:
    print(S[0])