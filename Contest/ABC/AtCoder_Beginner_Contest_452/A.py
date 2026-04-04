M, D = map(int,input().split())
L = [1,3,5,7,9]

if M in L:
    if M == 1 and D == 7:
        print("Yes")
    elif M != 1 and M == D:
        print("Yes")
    else:
        print("No")
else:
    print("No")