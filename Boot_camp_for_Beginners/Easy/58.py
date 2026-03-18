N = int(input())
T, A = map(int,input().split())
H = list(map(int,input().split()))

ans = float("inf")

for i in range(N):
    if abs(A-(T-H[i]*0.006)) < ans:
        ans_i = i+1
        ans = abs(A-(T-H[i]*0.006))

print(ans_i)