N, K = map(int,input().split())

cnt = 0

for i in range(N):
    temp = N-i
    D = list(map(int, str(temp)))

    if sum(D) == K:
        cnt += 1

print(cnt)