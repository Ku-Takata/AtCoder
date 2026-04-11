T, X = map(int,input().split())
A = list(map(int,input().split()))

print(0,A[0])
val = A[0]
for i in range(1,T+1):
    if abs(A[i] - val) >= X:
        print(i,A[i])
        val = A[i]