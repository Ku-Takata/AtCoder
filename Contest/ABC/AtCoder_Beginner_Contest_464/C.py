N, M = map(int,input().split())
ADB = [list(map(int,input().split())) for i in range(N)]

# やりたいこととしては、各鳥の数を辞書にして、ADBを日付順でソートして、N個のクエリを順にこなしていく
# 鳥の種類は単調減少
# WA??
# 鳥の数マイナスになってしまっているケースがあるのかこれ
# これも違う？？？？
# これ初期状態がそもそも各色で1羽いるわけじゃなくて、A_iが実際にいる色の鳥？

bird = dict()
for i in range(N):
    A = ADB[i][0]
    if A not in bird:
        bird[A] = 0
    bird[A] += 1
ADB.sort(key=lambda x:x[1])
ans = [N for i in range(M)]
total = len(bird)
i = 0

for d in range(1,M+1):
    while i < N and ADB[i][1] == d:
        A,D,B = ADB[i][0],ADB[i][1],ADB[i][2]
        if A != B and bird[A] > 0:
            if B not in bird:
                bird[B] = 0

            bird[B] += 1
            bird[A] -= 1
            if bird[A] == 0:
                total -= 1
            if bird[B] == 1:
                total += 1
        i += 1

    ans[d-1] = total

for i in range(M):
    print(ans[i])

"""
for i in range(N):
    A,D,B = ADB[i][0],ADB[i][1]-1,ADB[i][2]
    if i < N-1:
        next_D = ADB[i+1][1]
    else:
        next_D = M

    if A != B:
        bird[B] += 1
        bird[A] -= 1
        if bird[A] == 0:
            total -= 1
        if bird[B] == 1:
            total += 1

    for j in range(next_D - D):
        ans[D+j] = total

    # print(A,B)
    # print(bird)

# print(ADB)

for i in range(M):
    print(ans[i])

"""