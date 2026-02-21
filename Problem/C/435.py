N = int(input())
A = list(map(int,input().split()))

cnt = 1
tall = A[0]

# 次のが倒せる回数をカウント
for i in range(N-1):
    tall -= 1

    if tall == 0:
        break
    else:
        if tall < A[i+1]:
            tall = A[i+1]
    cnt += 1

print(cnt)

# Greedy algo, simulation