N = int(input())

word_list = []
ans = "Yes"

for i in range(N):
    word = input()
    if i == 0:
        word_list.append(word)
    elif word not in word_list and word_list[i-1][-1] == word[0]:
        word_list.append(word)
    else:
        ans = "No"
        break

print(ans)

# 全部一旦読み込ませないといけない場合は、これだとREになるが、AtCoderでは出力が合っていればOKなのでこれで通る。