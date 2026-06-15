N, A, B = map(int,input().split())

# A-Bが偶奇で違う
# 1かN卓に近い方を一旦足踏みさせたい

if (A+B) % 2 == 0:
    ans = abs(A-B) // 2
else:
    if min(A,B)-1 < N-max(A,B):
        ans = min(A,B) + (max(A,B)-min(A,B)-1) // 2
    else:
        ans = N-max(A,B)+1 + (max(A,B)-min(A,B)-1) // 2

print(ans)