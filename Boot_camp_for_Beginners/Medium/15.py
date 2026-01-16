S = list(input())
keyence = ["k", "e", "y", "e", "n", "c", "e"]
cnt_index = 0

# 先頭と末尾を合わせてKeyenceになれば良い
for i in range(len(S)):
    if S[i] == keyence[cnt_index]:
        cnt_index += 1
        if cnt_index == 7:
            renzoku = cnt_index
            break
    else:
        renzoku = cnt_index
        break

for i in range(len(S)):
    if renzoku == 7:
            break
    if S[len(S)-1-i] == keyence[-1-i]:
        renzoku += 1
    else:
        break

if renzoku == 7:
    print("YES")
else:
    print("NO")

# 78 そんなに難しい問題ではないはずだが、先頭と末尾だけを見たらよいという事に気づくのに１時間くらいかかってしまった。
# 気づいてからは18分程度で解けたので、問題文をよく読み、実装よりもまず確かな設計を考えることを癖付けたい。