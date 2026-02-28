S = list(input())
T = list(input())

S_rev = []
T_rev = []

for i in range(len(S)):
    if S[i] != "A":
        S_rev.append(S[i])

for i in range(len(T)):
    if T[i] != "A":
        T_rev.append(T[i])

# -1チェック
if S_rev != T_rev:
    print(-1)
    exit()

A_cnt = []
now = 0

for i in S:
    if i == "A":
        now += 1
    else:
        A_cnt.append(now)
        now = 0

A_cnt.append(now)

B_cnt = []
now = 0

for i in T:
    if i == "A":
        now += 1
    else:
        B_cnt.append(now)
        now = 0

B_cnt.append(now)

# print(A_cnt)
# print(B_cnt)

ans = 0

for i in range(len(A_cnt)):
    ans += abs(A_cnt[i]-B_cnt[i])

print(ans)