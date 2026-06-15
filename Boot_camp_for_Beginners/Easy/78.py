N = int(input())
H = list(map(int,input().split()))

if N == 1:
    print("Yes")
else:
    for i in range(1,N):
        if H[i-1] < H[i]:
            H[i] -= 1
        elif H[i-1] == H[i]:
            continue
        else:
            print("No")
            exit()
    print("Yes")