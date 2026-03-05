N, A, B = map(int,input().split())
C = list(map(int,input().split()))

wa = A+B

for i in range(N):
    if wa == C[i]:
        print(i+1)
        exit()