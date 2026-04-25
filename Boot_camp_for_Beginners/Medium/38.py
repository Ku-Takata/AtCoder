N = int(input())
D = list(map(int,input().split()))
M = int(input())
T = list(map(int,input().split()))

# Dの問題集にTの問題集が内包されているかどうかを判定
# 辞書型で順番に見ていけば良さそう

from collections import Counter

cnt_D = Counter(D)
cnt_T = Counter(T)

for k,v in cnt_T.items():
    if cnt_D[k] >= v:
        continue
    else:
        print("NO")
        exit()

print("YES")