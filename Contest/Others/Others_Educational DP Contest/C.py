N = int(input())
ABC = [list(map(int,input().split())) for i in range(N)]

dp_a = [0]*N
dp_b = [0]*N
dp_c = [0]*N

dp_a[0] = ABC[0][0]
dp_b[0] = ABC[0][1]
dp_c[0] = ABC[0][2]

for i in range(1,N):
    dp_a[i] = max(dp_b[i-1], dp_c[i-1]) + ABC[i][0]
    dp_b[i] = max(dp_a[i-1], dp_c[i-1]) + ABC[i][1]
    dp_c[i] = max(dp_a[i-1], dp_b[i-1]) + ABC[i][2]

print(max(dp_a[-1], dp_b[-1], dp_c[-1]))