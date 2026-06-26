N = int(input())
A = list(map(int,input().split()))

for i in range(N):
    A[i] = [i,A[i]]

A.sort(key=lambda x:x[1],reverse=True)
cnt = 0
ans = [[A[0][0],0]]

for i in range(N-1):
    if A[i+1][1] < A[i][1]:
        cnt = i+1
    ans.append([A[i+1][0],cnt])

# print(A)
# print(ans)
ans.sort(key=lambda x:x[0])

for i in range(N):
    ans[i] = ans[i][1]

print(*ans)