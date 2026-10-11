N = int(input())
S = [input() for i in range(N)]

folder = dict()

for s in set(S):
    folder[s] = 0

for s in S:
    if folder[s] > 0:
        print(f"{s}({folder[s]})")
    else:
        print(s)

    folder[s] += 1