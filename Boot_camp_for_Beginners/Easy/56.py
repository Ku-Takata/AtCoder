A,B,K = map(int,input().split())

num = []

for i in range(K):
    if A+i not in num and A+i <= B:
        num.append(A+i)
    if B-i not in num and B-i >= A:
        num.append(B-i)

num.sort()

for i in range(len(num)):
    print(num[i])