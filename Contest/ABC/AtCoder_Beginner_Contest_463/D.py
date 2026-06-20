N, K = map(int,input().split())
LR = [list(map(int,input().split())) for i in range(N)]

# つまるところK枚布を選んだ時にどうしてもある布が重なっていたら-1
# それ以外は距離が近い布同士の距離の最小値を最大化したい
# これって貪欲法で行けるんじゃない？
# Rの方が小さい値をなるべく選ぶようにしたら最大だと思う
# WAらしい。たぶんK枚選んだ時に右側にまだ余白があってもう少し間を空けて布を置けるときがあるから
# 例えば、[1,2],[3,4],[9,10]ってあって2枚選ぶとしたら、これだと最初の2枚を選んでしまってWA
# じゃあどうやって判定すんのって話
# 今ある最小値よりなるべく小さくならないように制御したい
# これあれか、傾向的に最小値を最大化したいからニブタンか

LR.sort(key=lambda x: x[1])
# print(LR)

def greedy(X):
    cnt = 1
    R = LR[0][1]

    for i in range(1,N):
        if cnt == K:
            break
        if R + X < LR[i][0]:
            cnt += 1
            R = LR[i][1]

    return cnt == K

# print(ans)

if not greedy(0):
    print(-1)
else:
    ok = 0
    ng = 10**9 + 10

    # mid以上ならまだ大きくできる可能性があって、できないなら小さくする
    while ng-ok > 1:
        mid = (ok+ng) // 2
        if greedy(mid):
            ok = mid
        else:
            ng = mid

    print(ok+1)