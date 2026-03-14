N, L, R = map(int,input().split())
S = list(input())

from collections import Counter, deque
import math
# 最初にRの範囲内にCounterで英小文字が何個あるのかカウント
cnt_S = Counter(S[L:R+1])

# その後L回を越えたら順番に消していって、該当の英小文字カウントを1減らす
# 最後の方は消さないように注意
# 順番に組み合わせの数を計算していく
# これで計算量は大丈夫なはず

que = deque(S[L:R+1])
len_q = len(que)
ans = 0
k = 0

# print(que)
# print(cnt_S)

for i in range(N):
    ans += cnt_S[S[i]]
    # print(ans)

    if i+R+1 < N:
        que.append(S[i+R+1])

    if len(que) > len_q:
        cnt_S[que[-1]] += 1
    if len(que) > 0:
        cnt_S[que.popleft()] -= 1
    # print(que)
    # print(cnt_S)

print(ans)