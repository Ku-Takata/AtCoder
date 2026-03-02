N, T = map(int,input().split())

time = 0

for i in range(N):
    A, C = map(int,input().split())
    if A < T:
        # 必要なテストの点数
        point = T-A
        # 必要時間
        time += point*C

print(time)