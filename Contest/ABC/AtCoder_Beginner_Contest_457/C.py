N, K = map(int,input().split())
A = [list(map(int,input().split()))[1:] for i in range(N)]
C = list(map(int,input().split()))

# 方針としては、Lの数の総和を取っていって、Kを越えた時点で終わって、B[K]の値を出力する

cnt = 0
i = 0

while cnt < K:
    cnt += C[i] * len(A[i])
    i += 1

pre_cnt = cnt - C[i-1] * len(A[i-1])
pos = K - pre_cnt
ans_idx = (pos - 1) % len(A[i-1])
print(A[i-1][ans_idx])