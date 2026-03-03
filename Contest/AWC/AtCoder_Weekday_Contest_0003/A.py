N, K = map(int,input().split())
cnt = 0

for i in range(N):
    A, B = map(int,input().split())
    if A*B >= K:
        cnt += 1

print(cnt)