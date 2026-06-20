N = int(input())
HL = [list(map(int,input().split())) for i in range(N)]
Q = int(input())
T = list(map(int,input().split()))

# つまりTi < max(Takahashi) < Ti+1を求めたい
# 時間が10**5までだから、各時間ごとに最大の高橋を置くリストを作りたい
"""
HL.sort(key=lambda x: x[1],reverse=True)
max_time = HL[0][1]
HL.sort(key=lambda x: x[0],reverse=True)
# print(HL,max_time)
takahashi = []
i,j = 0,0

for i in range(N):
    while j < HL[i][1] and j < max_time:
        takahashi.append(HL[i][0])
        j += 1

# print(takahashi)

for i in range(Q):
    print(takahashi[T[i]])
"""

# 時間10**9やんけ
# じゃあクエリ側をインデックス付きでソートして後でインデックス順に直せばいけるのでは？
HL.sort(key=lambda x: x[0],reverse=True)
for i in range(Q):
    T[i] = [T[i],i]
T.sort(key=lambda x: x[0])

# print(T)
# print(HL)
ans = [[T[i][1]] for i in range(Q)]
i,j = 0,0
# print(ans)
while i < Q:
    if T[i][0] < HL[j][1]:
        ans[i].append(HL[j][0])
        i += 1
    else:
        j += 1

# print(ans)

ans.sort(key=lambda x: x[0])

for i in range(Q):
    print(ans[i][1])