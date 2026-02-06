R,G,B,N = map(int,input().split())

cnt = 0
"""
for i in range((N//R)+1):
    for j in range((N//G)+1):
        if R*i + G*j <= N and (N - (R*i + G*j)) % B == 0:
            cnt += 1

print(cnt)
"""
# これだとNを越えていても増やし続けるループになっているので、そこそこ遅いが一応AC
# 以下だと越えたらBreakしているので、より高速

for i in range((N//R)+1):
    for j in range((N//G)+1):
        if R*i + G*j > N:
            break
        if (N - (R*i + G*j)) % B == 0:
            cnt += 1

print(cnt)