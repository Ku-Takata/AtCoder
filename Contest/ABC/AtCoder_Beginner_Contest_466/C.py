N = int(input())

# 何この初めての問題パターン
# 各点間の距離が1以下かどうかを聞いて、Yesのものの数をカウント
# ただし聞ける回数は2N回以下
# これって単調増加だよね
# だから尺取みたいに解けば良いんかな

i,j = 1,1
cnt = 0
flag = False

for _ in range(2*N):
    while i < N:
        while j < N:
            print(f"? {i} {j+1}", flush=True)
            res = input()

            if res != "Yes":
                break
            j += 1

        cnt += j - i
        i += 1
        if i > j:
            j = i

print(f"! {cnt}",flush=True)
exit()