A, B, C, X, Y = map(int,input().split())

# ABピザ <= Aピザ + Bピザ であるとき、ABピザをmin(X,Y)枚買う
# Aピザ or Bピザ <= ABピザ*2 であるとき、ABピザをそれぞれ必要枚数買う
# それ以外は普通に買う

ans = 0

if C*2 <= A+B:
    ans += C * min(X,Y) * 2
    minus = min(X,Y)
    X -= minus
    Y -= minus

if C*2 <= A:
    ans += C*2*X
    X = 0
else:
    ans += A*X

if C*2 <= B:
    ans += C*2*Y
    Y = 0
else:
    ans += B*Y

print(ans)