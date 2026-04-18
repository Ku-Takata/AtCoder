N, K = map(int,input().split())
A = list(map(int,input().split()))

# [2,1]だったら1つ、[2,1,2,1]だったら3つ、[2,1,2,1,2,1]だったら6つ
# それぞれの要素での転倒数をカウントしたリストは以下のとおり
# [0,1],[0,1,0,2],[0,1,0,2,0,3]
# [3,2,1],[3,2,1,3,2,1]だったら
# [0,1,2],[0,1,2,0,2,4]
# つまりその要素のカウント値分増え続けると予想
# 最初の転倒数カウントはN<=2000だから愚直にできそう

inv_cnt = []

for i in range(N):
    cnt = 0
    for j in range(0,i):
        if A[j] > A[i]:
            cnt += 1
    inv_cnt.append(cnt)

ans = 0

for i in range(N):
    ans += (inv_cnt[i] + inv_cnt[i]*K) / 2