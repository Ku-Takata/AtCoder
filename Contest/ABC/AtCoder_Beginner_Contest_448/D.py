N = int(input())
A = list(map(int,input().split()))

import sys
sys.setrecursionlimit(10**7)

def dfs(pos,):
    if A[pos] not in cnt_num:
        cnt_num[A[pos]] = 1
    else:
        cnt_num[A[pos]] += 1
        same = True

    


graph = [[] for i in range(N)]

# DFSかな？
# まず頂点同士を結び付けるクエリを実行
for i in range(N-1):
    U, V = map(int,input().split())

    graph[U-1].append(V-1)
    graph[V-1].append(U-1)

# kで同じ数字があるかどうかを記録する
# 経路での値の個数をカウントする
# 同じ数があったらTrueにして、ansをYesに変える
ans = ["No"]*N
cnt_num = {}
same = False

for i in range(N):
    print(ans[i])

