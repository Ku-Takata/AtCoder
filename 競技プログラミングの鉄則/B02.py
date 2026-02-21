A, B = map(int,input().split())

for n in range(A,B+1):
    if 100 % n == 0:
        print("Yes")
        exit()

print("No")