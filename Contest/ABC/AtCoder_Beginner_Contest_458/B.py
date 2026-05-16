H, W = map(int,input().split())

if H > 1 and W > 1:
    for i in range(H):
        ans = []
        for j in range(W):
            if i == 0 and j == 0:
                ans.append(2)
            elif i == 0 and j == W-1:
                ans.append(2)
            elif i == H-1 and j == 0:
                ans.append(2)
            elif i == H-1 and j == W-1:
                ans.append(2)
            elif i == 0 or i == H-1:
                ans.append(3)
            elif j == 0 or j == W-1:
                ans.append(3)
            else:
                ans.append(4)

        print(*ans)
elif H == 1 and W == 1:
    print(0)
else:
    for i in range(H):
        ans = []
        for j in range(W):
            if i == 0 and j == 0:
                ans.append(1)
            elif i == H-1 and j == W-1:
                ans.append(1)
            else:
                ans.append(2)

        print(*ans)