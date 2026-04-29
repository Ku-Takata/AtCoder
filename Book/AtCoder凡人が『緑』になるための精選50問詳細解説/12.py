A,B = map(int,input().split())

# 最小公倍数を求める
import math

def LCM(x,y):
    return (x*y)//math.gcd(x,y)

print(LCM(A,B))

# 最小公倍数の求め方を知らなかった(忘れた)ため調べた。
# 2つの数の積を最大公約数で割ったもので求まる。