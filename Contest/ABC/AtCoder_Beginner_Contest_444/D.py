N = int(input())
A = list(map(int,input().split()))

# Aは桁数を表す
from collections import Counter

cnt_A = Counter(A)
total = 0
"""
for k,v in cnt_A.items():
    B = int("1"*k)*v
    total += B

print(total)
"""

ans = [0]*(max(A)+5)

