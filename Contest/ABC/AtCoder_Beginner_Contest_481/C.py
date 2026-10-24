from collections import defaultdict

Q = int(input())
S = [input() for i in range(Q)]

bank_id = defaultdict(lambda:0)
now = 1

for s in S:
    if bank_id[s] == 0:
        bank_id[s] = now
        now += 1
        print(bank_id[s])
    else:
        print(bank_id[s])