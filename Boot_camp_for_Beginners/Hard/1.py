S = input()
N = len(S)

# 次のマスと今のマスが同じ文字列つまりS[i] == S[i+1]であるときは、今のマスの人数は0
# 連続した場合に、次のLRもしくはRLにたどり着くまでの移動回数が
# 奇数の場合は後ろ側のマスに移動する
# 偶数の場合は前側に移動する
# つまり連続しなくなったときに、それまで連続していた数//2の繰り上げが後ろ側、それ以外が前側

R_streak = []
L_streak = []
streak = 0

for i in range(N):
    if i < N-1:
        if S[i] == S[i+1] and S[i] == "R":
            streak += 1
            R_streak.append(-1)
        else:
            R_streak.append(streak)
            streak = 0
    else:
        R_streak.append(streak)

for i in range(N-1,-1,-1):
    if i > 0:
        if S[i] == S[i-1] and S[i] == "L":
            streak += 1
            L_streak.append(-1)
        else:
            L_streak.append(streak)
            streak = 0
    else:
        L_streak.append(streak)

# print(R_streak)
# print(L_streak)

from math import ceil
ans = [1 for i in range(N)]

for i in range(N):
    if i < N-1:
        if R_streak[i] == -1:
            ans[i] = 0
        elif R_streak[i] > 0:
            ans[i+1] += ceil(R_streak[i]/2)
            ans[i] += R_streak[i] - ceil(R_streak[i]/2)
    else:
        if R_streak[i] > 0:
            ans[i] += R_streak[i]

# print(ans)

for i in range(N):
    if i < N-1:
        if L_streak[i] == -1:
            ans[-i-1] = 0
        elif L_streak[i] > 0:
            ans[-i-2] += ceil(L_streak[i]/2)
            ans[-i-1] += L_streak[i] - ceil(L_streak[i]/2)
    else:
        if L_streak[i] > 0:
            ans[-i-1] += L_streak[i]

print(*ans)

# 1時間くらいかかったけど今解けるDiff+100~200くらいの問題が解けた。シンプル嬉しい。