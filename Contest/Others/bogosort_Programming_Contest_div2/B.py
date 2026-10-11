T = int(input())

# 前と後ろの値以上の値にのみ-1できる
# つまり最初の値と最後の値は必然的に操作不可
# 左の値より右の値が大きいもしくは等しいなら単調非減少にできる

for i in range(T):
    N = int(input())
    A = list(map(int,input().split()))

    if A[0] > A[-1]:
        print(-1)
        continue

    now = A[-1]
    ans = 0

    for i in range(N-1,0,-1):
        if A[i] < A[0]:
            ans = -1

    if ans:
        print(ans)
        continue

    for i in range(N-1,0,-1):
        if A[i] > now:
            ans += A[i] - now
            A[i] = now
        else:
            now = A[i]

    # print(A)
    print(ans)