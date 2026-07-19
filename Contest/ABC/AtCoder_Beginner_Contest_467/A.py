H, W = map(int,input().split())

if W / ((H**2)/10000) >= 25:
    print("Yes")
else:
    print("No")