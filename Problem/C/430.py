N, A, B = map(int,input().split())
S = input()

# 尺取法っぽさある

cnt_a,cnt_b = 0,0
L_a,L_b = 0,0
ans = 0

for R in range(N):
    # 右端にある文字をカウント
    if S[R] == "a":
        cnt_a += 1
    else:
        cnt_b += 1

    # bがB個未満になるまで左端を進める
    while cnt_b >= B:
        if S[L_b] == "b":
            cnt_b -= 1
        L_b += 1

    # aがA個になるまで左端を進める
    while L_a < R and cnt_a >= A:
        if S[L_a] == "b":
            L_a += 1
        elif S[L_a] == "a" and cnt_a > A:
            cnt_a -= 1
            L_a += 1
        else:
            break

    if cnt_a >= A:
        if L_b <= L_a:
            ans += L_a - L_b + 1

print(ans)