X = int(input())
Q = int(input())
board = [X]

# 毎回中央値探索はできない
# 中央値がどのように変わっていくかを考えると、
# 現段階の中央値よりも大きい値が2つなら前の中央値が左にシフト
# 小さい値2つなら前の中央値が右にシフト
# 大きいのと小さいのなら変化なし
# つまり一旦全部のABをリストにまとめて、逆順で中央値よりも大きかったか小さかったかを判定したらいけそう
# WA???
# 手元で確認したら同じ値かつ小さい値や大きい値が来たときにおかしくなってそう
# というかこれ優先度付キューで解けそう

AB = [list(map(int,input().split())) for i in range(Q)]
for a,b in AB:
    board.append(a)
    board.append(b)

board.sort()
print(board)
print(AB)
median_i = len(board)//2
ans = []

for i in range(Q-1,-1,-1):
    median = board[median_i]
    # print(median)
    if median <= AB[i][0] and median <= AB[i][1]:
        median_i -= 1
    elif median >= AB[i][0] and median >= AB[i][1]:
        median_i += 1

    ans.append(median)

for i in range(Q-1,-1,-1):
    print(ans[i])