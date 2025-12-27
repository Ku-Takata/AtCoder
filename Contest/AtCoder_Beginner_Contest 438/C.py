N = int(input())
A = list(map(int, input().split()))
k = 0

while k+3 < len(A):
    if 0 <= k <= len(A):
        if A[k] == A[k+1] == A[k+2] == A[k+3]:
            del A[k:k+4]
            k = 0
        else:
            k += 1

# ぷよぷよみたいに一回で複数消さないとTLEになると予想



print(len(A))