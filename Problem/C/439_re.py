N = int(input())

# (1,2), (1,3), (2,3), (1,4), (2,4), (3,4), (1,5) ...
# yが1増えるごとにxを1からy-1の値まで増やした組み合わせが存在する
# それをN以下まで列挙する
# 計算量はN**1/2で済むはず

ans = []

for y in range(2,int((10**7)**0.5)+1):
    if y**2 >= N:
        break
    for x in range(1,y):
        calc = x**2 + y**2
        if calc <= N:
            ans.append(calc)
        else:
            break

ans = sorted(ans)
dup = 0
ans_rev = []

for i in range(len(ans)-1):
    if ans[i] == ans[i+1] or ans[i] == dup:
        dup = ans[i]
    else:
        ans_rev.append(ans[i])

if len(ans) > 0 and dup != ans[-1]:
    ans_rev.append(ans[-1])

print(len(ans_rev))
print(*ans_rev)

# すごい面倒な処理をしていると思う。普通にカウント辞書でやったら良かった。