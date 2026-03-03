N, M = map(int,input().split())
if N % 2 == 0:
    N //=2
else:
    N = N//2 + 1

if M <= N:
    print("Yes")
else:
    print("No")