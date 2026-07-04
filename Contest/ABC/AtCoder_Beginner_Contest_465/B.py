X, Y, L, R, A, B = map(int,input().split())

fee = 0

for time in range(A,B):
    if L <= time < R:
        fee += X
    else:
        fee += Y

print(fee)