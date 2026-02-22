N = int(input())
trip = {}

for i in range(N):
    t, x, y = map(int, input().split())
    trip[i] = x, y
    if trip[i] == (trip[i-1][0] + 1, trip[i-1][1]) or (trip[i-1][0] - 1, trip[i-1][1]) or (trip[i-1][0], trip[i-1][1] + 1) or (trip[i-1][0], trip[i-1][1] - 1):
        print("YES")
    else:
        print("NO")
