N = int(input())
P = list(map(int,input().split()))

min_num = float("inf")
cnt = 0

for i in range(N):
    if P[i] <= min_num:
        min_num = P[i]
        cnt += 1

print(cnt)