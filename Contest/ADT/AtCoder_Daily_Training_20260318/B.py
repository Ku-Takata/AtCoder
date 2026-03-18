H, W = map(int,input().split())
R, C = map(int,input().split())

cnt = 0

if R+1 <= H:
    cnt += 1
if R-1 > 0:
    cnt += 1
if C+1 <= W:
    cnt += 1
if C-1 > 0:
    cnt += 1

print(cnt)