N = int(input())
A = list(map(int,input().split()))

ans = 0

# 要素をインデックスに合わせる
for i in range(N):
    A[i] -= 1

for i in range(N):
    if i == A[A[i]] and i < A[i]:
        ans += 1

print(ans)