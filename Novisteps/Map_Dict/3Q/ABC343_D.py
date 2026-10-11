from collections import defaultdict

N, T = map(int,input().split())
AB = [list(map(int,input().split())) for i in range(T)]

scores = [0]*N
cnt = defaultdict(int)
cnt[0] = N

for a,b in AB:
    cnt[scores[a-1]] -= 1

    if cnt[scores[a-1]] == 0:
        del cnt[scores[a-1]]

    scores[a-1] += b
    cnt[scores[a-1]] += 1

    print(len(cnt))