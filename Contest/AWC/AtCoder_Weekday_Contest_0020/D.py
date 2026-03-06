N, L = map(int,input().split())
XR = [list(map(int,input().split())) for i in range(N)]

# Xの座標にかかわらず、Rの範囲による
XR.sort(key=lambda x: x[0] - x[1])

# 被っている範囲を上手く処理しないといけない
# カバー範囲の上限まで都度歩き進めるのが良さそう

now = 0

for i in range(N):
    if XR[i][0]-XR[i][1] <= now:
        now = max(XR[i][0]+XR[i][1], now)
    # print(now)
if now >= L:
    print("Yes")
else:
    print("No")