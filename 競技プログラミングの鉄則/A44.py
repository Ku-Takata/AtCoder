N, Q = map(int,input().split())
A = [i for i in range(1,N+1)]

reverse = False
for i in range(Q):
    query = list(map(int,input().split()))

    if query[0] == 3:
        if reverse:
            print(A[N-query[1]])
        else:
            print(A[query[1]-1])
    elif query[0] == 2:
        if reverse:
            reverse = False
        else:
            reverse = True
    else:
        if reverse:
            A[N-query[1]] = query[2]
        else:
            A[query[1]-1] = query[2]
    # print(A,reverse)