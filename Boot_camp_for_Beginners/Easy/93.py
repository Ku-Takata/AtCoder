N = int(input())
A = list(map(int,input().split()))

# とりあえず棒の長さが偶数じゃないと2等分できない
# 右左に分けて同値にするようにしたらいい

L,R = 0,sum(A)
# diff = []
ans = float("inf")

for i in range(N):
    L += A[i]
    R -= A[i]
    # diff.append(abs(L-R))
    ans = min(ans,abs(L-R))

# print(min(diff))
print(ans)