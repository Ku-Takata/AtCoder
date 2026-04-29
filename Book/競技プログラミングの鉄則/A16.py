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
print(dp[-1])