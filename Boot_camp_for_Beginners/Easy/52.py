N = int(input())
B = list(map(int,input().split()))

A = []

for i in range(N):
    if i == 0:
        A.append(B[0])
    elif i == N-1:
        A.append(B[-1])
    else:
        A.append(min(B[i-1],B[i]))

print(sum(A))

# i = 1のときもB[0]を入れてしまっていた。例を考えたら無くすのが正解だと分かったが、結構つまづいた。