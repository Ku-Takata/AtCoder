N, K = map(int,input().split())

manzoku = 0

for i in range(N):
    C, D = map(int,input().split())

    if C <= K:
        manzoku += D

print(manzoku)