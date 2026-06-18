D, N = map(int,input().split())

if D == 0:
    if N != 100:
        print(N)
    else:
        print(N+1)
else:
    if N != 100:
        print(100**D*N)
    else:
        print(100**D*N + 100**D)