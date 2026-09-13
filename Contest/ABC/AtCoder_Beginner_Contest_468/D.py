S = input()

# 1文字書き換えて回文になる文字列を見つける問題
# DPで部分文字列が何回操作して回文になるかを記録していくのが良さそう
# 文字列の長さ2までは確定で回文、3以上で操作によって回文になるか考える

N = len(S)

# dp[i][j]は部分文字列S[i:j]を回文にするための最小操作回数
dp = [[0]*(N + 1) for i in range(N + 1)]

ans = 0

for len_sub in range(1,N+1):
    for i in range(N - len_sub + 1):
        j = i + len_sub

        if len_sub <= 2:
            if S[i] != S[j-1]:
                dp[i][j] = 1
        else:
            if S[i] != S[j-1]:
                mismatch = 1
            else:
                mismatch = 0

            dp[i][j] = dp[i+1][j-1] + mismatch

        if dp[i][j] <= 1:
            ans += 1

print(ans)
