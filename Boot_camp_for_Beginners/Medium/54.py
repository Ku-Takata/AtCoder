N, C, K = map(int,input().split())
T = [int(input()) for i in range(N)]

# N人の乗客をなるべくギリギリまで待たせてバスに乗せたい

T.sort()
time = 0
cnt = 0
ans = 0

for t in T:
    if t <= time and cnt < C:
        cnt += 1
        # print(time,cnt,ans)
        continue
    time = t + K
    cnt = 1
    ans += 1
    # print(time,cnt,ans)

print(ans)