N = int(input())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

# 勇者はi番目と次の街のモンスターしか倒せない
# 貪欲にi番目のモンスターを倒して、その後に次の街のモンスターを倒したらいいのでは？

ans = 0

for i in range(N):
    if B[i] <= A[i]:
        ans += B[i]
        A[i] -= B[i]
        B[i] = 0
    else:
        ans += A[i]
        B[i] -= A[i]
        A[i] = 0

    if B[i] > 0:
        if B[i] <= A[i+1]:
            ans += B[i]
            A[i+1] -= B[i]
            B[i] = 0
        else:
            ans += A[i+1]
            B[i] -= A[i+1]
            A[i+1] = 0

print(ans)