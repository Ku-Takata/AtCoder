N, M, X = map(int,input().split())

C = []
A = []

for i in range(N):
    book = list(map(int,input().split()))
    C.append(book[0])
    A.append(book[1:])

ans = float("inf")

for i in range(1<<N):
    cost = 0
    algo = [0]*M

    for shift in range(N):
        if i>>shift & 1 == 1:
            cost += C[shift]
            for j in range(M):
                algo[j] += A[shift][j]
    if min(algo) >= X:
        ans = min(ans,cost)

if ans == float("inf"):
    print(-1)
else:
    print(ans)