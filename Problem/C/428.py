Q = int(input())

# "("で終わったら確定でNo、")"で終わったときにこれまでの"("と")"の数が一致したらYes

cnt = 0
bad_cnt = 0
S = []

for i in range(Q):
    query = list(map(str,input().split()))

    if query[0] == "1":
        if query[1] == "(":
            cnt += 1
        else:
            cnt -= 1
            if cnt < 0:
                bad_cnt += 1

        S.append(cnt)
    else:
        removed_cnt = S.pop()

        if removed_cnt < 0:
            bad_cnt -= 1

        if len(S) == 0:
            cnt = 0
        else:
            cnt = S[-1]

    if cnt == 0 and bad_cnt == 0:
        print("Yes")
    else:
        print("No")