N, M = map(int,input().split())
order = []

for i in range(N):
    L = int(input())
    X = list(map(int,input().split()))

    for j in range(L):
        if X[j] not in order:
            order.append(X[j])
            print(X[j])
            break
        elif j == L-1 and X[j] in order:
            print(0)