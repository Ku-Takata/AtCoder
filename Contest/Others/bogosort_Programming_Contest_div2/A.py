N = int(input())

while True:
    P = list(map(int,input().split()))
    if P == sorted(P):
        print("ok")
        exit()
    else:
        print("shuffle")