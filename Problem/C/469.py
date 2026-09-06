N = int(input())
S = input()

total = 0
hit = 0
i = 0

for _ in range(1,N+1):
    hit = 1

    while hit > 0:
        hit -= 1
        total += 1
        if S[i] == "o":
            hit += 1
        else:
            hit -= 1

        if i < N-1:
            i += 1
        else:
            break

    if total >= N:
        print(N)
    else:
        print(total)