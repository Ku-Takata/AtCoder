A = [list(map(int,input().split())) for i in range(3)]

# 組合わせは6通り、456, 465, 546, 564, 645, 654

ans = 0

for first in [4,5,6]:
    cnt_first = A[0].count(first)
    for second in [4,5,6]:
        if first == second:
            continue
        cnt_second = A[1].count(second)
        for third in [4,5,6]:
            if third == second or third == first:
                continue
            cnt_third = A[2].count(third)

            # 確率計算
            if cnt_first != 0:
                prob_f = cnt_first / 6
            else:
                continue

            if cnt_second != 0:
                prob_s = cnt_second / 6
            else:
                continue

            if cnt_third != 0:
                prob_t = cnt_third / 6
            else:
                continue

            ans += prob_f * prob_s * prob_t

print(ans)