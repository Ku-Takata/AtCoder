N, M = map(int,input().split())
S = input()
T = input()
Q = int(input())

for i in range(Q):
    w = input()
    takahashi = True
    aoki = True

    for j in range(len(w)):
        if w[j] not in S:
            takahashi = False
        if w[j] not in T:
            aoki = False

    if takahashi == aoki:
        print("Unknown")
    elif takahashi:
        print("Takahashi")
    else:
        print("Aoki")