N = int(input())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

for i in range(N):
    if B[A[i]-1] == i+1:
        continue
    else:
        print("No")
        exit()

print("Yes")