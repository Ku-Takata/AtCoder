A, B = map(int,input().split())
A_copy = A
B_copy = B
while A != 0 and B != 0:
    if A > B:
        A %= B
    else:
        B %= A

# LCMはGCD*LCM = A*Bが成り立つ
LCM = A_copy*B_copy//max(A,B)
print(LCM)