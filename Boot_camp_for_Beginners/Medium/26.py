N = int(input())
A = list(map(int,input().split()))

cnt = 0
stat = 0

for i in range(N-1):
    if A[i] < A[i+1]:
        if stat == "minus":
            cnt += 1
            stat = 0
            continue
        stat = "plus"
    elif A[i] > A[i+1]:
        if stat == "plus":
            cnt += 1
            stat = 0
            continue
        stat = "minus"

print(cnt+1)