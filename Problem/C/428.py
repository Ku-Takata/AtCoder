Q = int(input())

# "("で終わったら確定でNo、")"で終わったときにこれまでの"("と")"の数が一致したらYes

cnt = 0
min_cnt = 0
S = []

for i in range(Q):
    query = list(map(str, input().split()))

    if query[0] == "1":
        if query[1] == "(":
            cnt += 1
        else:
            cnt -= 1

        if len(S) == 0:
            prev_min = 0
        else:
            prev_min = S[-1][1]

        min_cnt = min(prev_min, cnt)
        S.append((cnt, min_cnt))

    else:
        S.pop()
        if len(S) == 0:
            cnt = 0
            min_cnt = 0
        else:
            cnt = S[-1][0]
            min_cnt = S[-1][1]

    if cnt == 0 and min_cnt >= 0:
        print("Yes")
    else:
        print("No")