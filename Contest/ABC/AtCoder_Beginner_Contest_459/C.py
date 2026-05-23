N, Q = map(int,input().split())

# マス全てにブロックが積まれたと判定したいので、辞書で管理したら良さそう
# 何個以上積まれているかも辞書が良さそう
# 全部積まれたら出力時に―1したら良い

from collections import Counter

blocks = [0]*N
cnt_blocks = Counter({0:N})
base = 0
cnt = 0

# 毎回合計しているとTLEになるので、もっと軽くする
max_blocks = Q+2
least_b = [0]*max_blocks
least_b[0] = N

for i in range(Q):
    q, num = map(int,input().split())

    if q == 1:
        now = blocks[num-1]
        # ブロック数が0だったマスが1に増えるか
        if now - base == 0:
            cnt += 1

        cnt_blocks[now] -= 1
        blocks[num-1] += 1
        cnt_blocks[blocks[num-1]] += 1

        next_b = blocks[num-1]
        least_b[next_b] += 1

        # 全マスにブロックが積まれたら、基準値を1上げる
        if cnt == N:
            base += 1
            # cnt = sum(cnt_b for b, cnt_b in cnt_blocks.items() if b > base)

            if base+1 < max_blocks:
                cnt = least_b[base+1]
            else:
                cnt = 0

    elif q == 2:
        # 基準値baseを足し合わせてマスをカウントする
        y = num + base
        if y < max_blocks:
            ans = least_b[y]
        else:
            ans = 0

        print(ans)