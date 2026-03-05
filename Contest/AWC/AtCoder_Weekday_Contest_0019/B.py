N, K = map(int,input().split())

cnt = 0

for i in range(N):
    S = input()
    if S.count("!") >= K:
        cnt += 1

print(cnt)