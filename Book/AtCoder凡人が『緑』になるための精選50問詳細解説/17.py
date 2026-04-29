N = int(input())
ab_list = []

for a in range(2, (10**5)+1):
    for b in range(2,50):
        if a ** b <= 10**10:
            ab_list.append(a ** b)
        else:
            break

ab_list = list(set(ab_list))

# 102719 ≒ 10^5
# print(len(ab_list))

cnt = 0

for i in range(len(ab_list)):
    if ab_list[i] <= N:
        cnt += 1

print(N-cnt)

# 20 特に難しい点はなかったが、問題文のa,bに制約が無い事に気づくのに時間がかかった。
# 以下の方が小さい数に対して効果的なので、早くなると思ったが、テストケースが大きい数字ばかりらしく、逆にソートの分遅くなった。
"""
ab_list = sorted(list(set(ab_list)))

for i in range(len(ab_list)):
    if ab_list[i] <= N:
        cnt += 1
    else:
        break
"""