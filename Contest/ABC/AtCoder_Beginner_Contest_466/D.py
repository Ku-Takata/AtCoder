N, M = map(int,input().split())
RC = [list(map(int,input().split())) for i in range(M)]

# ナンプレみたいやな
# 最終出力はコマの数
# 最大はN個、最小は1個
# N行とN列でそれぞれコマがあるかどうかを管理する
# 順番にやると、R行C列を見るのにO(N)かかるからO(NM)になるからTLEになりそう
# 逆順ならFalse,True操作が1回で済むからO(N+M)?
# とりあえず、逆なら後先考えずに操作できるから逆の方が圧倒的に早いはず

row = [False]*(N+1)
col = [False]*(N+1)
cnt = 0

for i in range(M-1,-1,-1):
    R,C = RC[i]

    if not row[R] and not col[C]:
        cnt += 1
    row[R] = True
    col[C] = True

print(cnt)