N = int(input())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

dp = []

for i in range(1,N+1):
    if i == 1:
        dp.append(0)
    elif i == 2:
        dp.append(A[0])
    else:
        if dp[i-3] + B[i-3] < dp[i-2] + A[i-2]:
            dp.append(dp[i-3] + B[i-3])
        else:
            dp.append(dp[i-2] + A[i-2])

# print(dp)
i = len(dp)-1
P = [i+1]

while i != 0:
    if dp[i] == dp[i-1] + A[i-1]:
        P.append(i)
        i -= 1
    else:
        P.append(i-1)
        i -= 2

print(len(P))
print(*reversed(P))