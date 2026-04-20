S = input()

# 難しすぎん？これ本当にDiff茶色以下か？
# これs1 != s2であれば良くて、全体で被りが１つもない状態にする必要はないのか

dp = [[-1]*3 for i in range(len(S)+1)]

if len(S) >= 1:
    dp[1][1] = 1
if len(S) >= 2:
    dp[2][2] = 1

for i in range(1,len(S)+1):
    for L in [1,2]:
        if i-L < 0:
            continue

        cur_str = S[i-L:i]

        for preL in [1,2]:
            if i-L-preL < 0:
                continue
            if dp[i-L][preL] == -1:
                continue

            pre_str = S[i-L-preL:i-L]

            if cur_str != pre_str:
                dp[i][L] = max(dp[i][L], dp[i - L][preL] + 1)

print(max(dp[len(S)][1], dp[len(S)][2]))