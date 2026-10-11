N, Q = map(int,input().split())
L = list(map(int,input().split()))
R = list(map(int,input().split()))

# かかるコストは今いるマスに依存で左か右か
# 直行か経由した方がいいのか問題
# LとRが気持ち悪いから合わせたい

L = [0,0] + L
R = [0] + R

min_R = [float("inf")] * (N+1)
for i in range(1,N):
    min_R[i] = min(min_R[i-1], R[i])

# print(min_R)

min_L = [float("inf")] * (N+2)
for i in range(N,1,-1):
    min_L[i] = min(min_L[i+1], L[i])

# print(min_L)

for i in range(Q):
    s, t = map(int,input().split())

    if s < t:
        ans = R[s]
        if s > 1:
            ans = min(R[s], L[s]+min_R[s-1])
    else:
        ans = L[s]
        if s < N:
            ans = min(L[s], R[s]+min_L[s+1])

    print(ans)