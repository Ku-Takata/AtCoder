N = int(input())
AB = [list(map(int,input().split())) for i in range(N)]

# 表の正の整数で揃えた場合
A,B = 0,0
for i in range(N):
    if AB[i][0]+AB[i][1] > 0:
        A += AB[i][0]
        B += AB[i][1]

ans = abs(A)+abs(B)


# 表の負の整数で揃えた場合
A,B = 0,0
for i in range(N):
    if AB[i][0]-AB[i][1] > 0:
        A += AB[i][0]
        B += AB[i][1]

if abs(A)+abs(B) > ans:
    ans = abs(A)+abs(B)

# 裏の正の整数で揃えた場合
A,B = 0,0
for i in range(N):
    if -AB[i][0]+AB[i][1] > 0:
        A += AB[i][0]
        B += AB[i][1]

if abs(A)+abs(B) > ans:
    ans = abs(A)+abs(B)

# 裏の負の整数で揃えた場合
A,B = 0,0
for i in range(N):
    if -AB[i][0]-AB[i][1] > 0:
        A += AB[i][0]
        B += AB[i][1]

if abs(A)+abs(B) > ans:
    ans = abs(A)+abs(B)

print(ans)