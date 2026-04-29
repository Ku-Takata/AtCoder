N = int(input())
S = list(input())

for i in range(N):
    if i == 0:
        now = S[i]
        cnt = 1
        continue

    if now == S[i]:
        cnt += 1
    else:
        cnt = 1
        now = S[i]

    if cnt == 3:
        print("Yes")
        exit()

print("No")