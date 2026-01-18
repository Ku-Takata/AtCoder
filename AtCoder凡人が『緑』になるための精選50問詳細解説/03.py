N = int(input())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

total = 0

for i in range(N):
    total += A[i]*B[i]

if total == 0:
    print("Yes")
else:
    print("No")

# 2 リストの基本操作