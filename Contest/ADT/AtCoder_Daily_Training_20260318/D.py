X, Y, Z = map(int,input().split())

if X < 0 and 0 > Y > X and Y > Z:
    print(-1)
elif X > 0 and 0 < Y < X and Y < Z:
    print(-1)
else:
    if X < 0 and (Y < X or Y > 0):
        print(abs(X))
    elif X > 0 and (Y > X or Y < 0):
        print(abs(X))
    elif X < 0 and X <= Y < 0:
        if Z < 0:
            print(abs(X))
        else:
            print(abs(X)+abs(Z)*2)
    elif X > 0 and X >= Y > 0:
        if Z > 0:
            print(abs(X))
        else:
            print(abs(X)+abs(Z)*2)