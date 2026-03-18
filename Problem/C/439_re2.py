N = int(input())

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

from collections import Counter
cnt_ans = Counter(ans)
ans_rev = []
ans = sorted(ans)

for i in range(len(ans)):
    if cnt_ans[ans[i]] == 1:
        ans_rev.append(ans[i])

print(len(ans_rev))
print(*ans_rev)

# 辞書版。解きやすくはあったけど時間はぎりぎり。