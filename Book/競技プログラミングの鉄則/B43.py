N, M = map(int,input().split())
A = list(map(int,input().split()))

incorrect = {i:0 for i in range(1,N+1)}

for i in range(M):
    incorrect[A[i]] += 1

for v in incorrect.values():
    print(M-v)