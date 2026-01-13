A = []

for t in range(3):
    A.append(list(map(int, input().split())))

N = int(input())

for n in range(N):
    b = int(input())
    A = [[("o" if b == a2 else a2) for a2 in a] for a in A]

# 確認用
"""for _ in range(3):
    print(A[_])"""

ans = "No"

for i in range(3):
    if A[i][0] == A[i][1] == A[i][2] == "o":
        ans = "Yes"
    elif A[0][i] == A[1][i] == A[2][i] == "o":
        ans = "Yes"

if A[0][0] == A[1][1] == A[2][2] == "o":
    ans = "Yes"
if A[0][2] == A[1][1] == A[2][0] == "o":
    ans = "Yes"

print(ans)