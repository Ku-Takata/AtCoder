N = int(input())
A = list(map(int,input().split()))

ameba = {1:0}

for i in range(N):
    child = ameba[A[i]] + 1

    ameba[2*(i+1)] = child
    ameba[2*(i+1)+1] = child

# print(ameba)
for j in range(1, 2*N + 2):
    print(ameba[j])