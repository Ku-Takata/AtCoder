A,B = map(int,input().split())

cnt = 0

for i in range(B-A+1):
    temp = str(B - i)
    if temp[0] == temp[4] and temp[1] == temp[3]:
        cnt += 1

print(cnt)