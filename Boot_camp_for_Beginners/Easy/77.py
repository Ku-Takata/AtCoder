N = int(input())
A = list(map(int,input().split()))

# 最大から二人、最小から一人のチームを作ったら良さそう
# だから結局1つ飛ばしで足し合わせたら最大じゃない？

A.sort(reverse=True)
ans = 0

for i in range(N):
    ans += A[2*i+1]

print(ans)