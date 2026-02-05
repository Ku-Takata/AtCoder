A,B,C,K = map(int,input().split())

# KがMax 10**18まであるのでTLE
"""
for i in range(K):
    tempA = B + C
    tempB = A + C
    tempC = B + A
    A,B,C = tempA,tempB,tempC

ans = A - B

if abs(ans) > 10**18:
    print("Unfair")
else:
    print(ans)
"""

if K % 2 == 0:
    print(A-B)
else:
    print(B-A)

# すごいシンプル。