H, W, Q = map(int,input().split())
for i in range(Q):
    q1, q2 = map(int,input().split())

    if q1 == 1:
        print(q2*W)
        H -= q2
    else:
        print(q2*H)
        W -= q2
