N = int(input())
S = []

for i in range(N):
    S.append(int(input()))

S = sorted(S)
total = sum(S)

if total % 10 == 0:
    for i in range(N):
        total = sum(S) - S[i]
        if total % 10 != 0:
            print(total)
            exit()

if total % 10 == 0:
    print(0)
else:
    print(total)

# 10 Mediumを10分で解けたのは初かもしれない。