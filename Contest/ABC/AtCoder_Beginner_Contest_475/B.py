N = int(input())
A = list(map(int,input().split()))

cnt1,cnt2,cnt3 = 0,0,0

for a in A:
    temp = divmod(a,1000)

    if temp[1] != 0:
        otsuri = 1000-temp[1]
    else:
        otsuri = 0

    temp = divmod(otsuri,100)
    otsuri = temp[1]
    cnt1 += temp[0]
    temp = divmod(otsuri,10)
    otsuri = temp[1]
    cnt2 += temp[0]
    temp = divmod(otsuri,1)
    otsuri = temp[1]
    cnt3 += temp[0]

print(cnt3,cnt2,cnt1)