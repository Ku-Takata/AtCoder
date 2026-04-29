Q = int(input())
X = [int(input()) for i in range(Q)]

# 今現在知っている√xまでを順番に割るアルゴリズム
for i in range(Q):
    j = 2
    ans = "Yes"
    while j <= X[i]**(1/2):
        if X[i] % j == 0:
            ans = "No"
            break
        else:
            j += 1
    print(ans)
