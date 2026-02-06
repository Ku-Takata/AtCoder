N, K = map(int,input().split())
A = list(map(int,input().split()))

from collections import Counter

cnt_A = Counter(A)
val_list = sorted(list(cnt_A.values()))
ans = 0

for i in range(len(set(A))-K):
    ans += val_list[i]

print(ans)

# 初めはループにリスト操作を行っており1つTLEになっていた。基本的にはリスト操作は計算量が増えるので、しなくていいならやめようと肝に銘じた。