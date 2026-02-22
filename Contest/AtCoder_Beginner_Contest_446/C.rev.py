T = int(input())
for i in range(T):
    N, D = map(int,input().split())
    A = list(map(int,input().split()))
    B = list(map(int,input().split()))

    syuusi = []

    for j in range(N):
        syuusi.append(A[j]-B[j])

    print(syuusi)

# D日過ぎた古い卵を廃棄する
# 多分貪欲法、苦手


"""
egg = [A[0]-B[0], 0]

    for j in range(N):
        egg[0] -= B[j]
        if egg[0] < 0:
            egg[0] += A[j]
        else:
            egg[1] += A[j]

        if j >= D-1:
            egg[0] = B[j]
        print(egg)
"""

"""
    cs = [0]

    for j in range(N):
        cs.append(cs[-1]+A[j]-B[j])

    print(cs)

    for j in range(N):
"""