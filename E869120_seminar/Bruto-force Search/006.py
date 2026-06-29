N = int(input())
S = input()

# パスワードの組み合わせを辞書で管理する
# 同じ数字の組み合わせが存在する
# 例えば、11111123とかなら、最初の1だけ見ておけばよくて、その後の1は見る必要がない
# これなら10*10*30000で3*10**6で済む

first = dict()
ans = 0

for i in range(N-2):
    second = dict()
    if first.get(S[i]) is None:
        first[S[i]] = 1
    else:
        continue
    for j in range(i+1,N-1):
        third = dict()
        if second.get(S[j]) is None:
            second[S[j]] = 1
        else:
            continue
        for k in range(j+1,N):
            if third.get(S[k]) is None:
                third[S[k]] = 1
            else:
                continue

        ans += len(third)

    # print(first,second,third)
print(ans)