N, M = map(int,input().split())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

dict_A = {}

for i in range(N):
    dict_A[i+1] = A[i]

for i in range(M):
    dict_A[B[i]] += 1
    if B[i] != N:
        dict_A[B[i]+1] += 1
    if B[i] != 1:
        dict_A[B[i]-1] += 1

for k,v in dict_A.items():
    A[k-1] = v

print(*A)