N, Q = map(int,input().split())
A = sorted(list(map(int,input().split())), reverse=True)
XY_list = []

for i in range(Q):
    XY = list(map(int,input().split()))
    XY_list.append(XY)

for x,y in XY_list:
    cnt = 0
    rekkyo = []
    x_up = []
    # x以上リストを作る
    for i in range(N):
        if A[i] >= x:
            x_up.append(A[i])
        else:
            break

    print(x_up)
