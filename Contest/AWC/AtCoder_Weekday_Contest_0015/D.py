N, M, C = map(int,input().split())
A = sorted(list(map(int,input().split())))
B = sorted(list(map(int,input().split())))

i,j = 0,0
cnt = 0

while i < N and j < M:
    # print(A[i],B[j])
    if A[i] >= B[j]:
        cnt += 1
        i += 1
        j += 1
    else:
        i += 1

print(cnt*C)