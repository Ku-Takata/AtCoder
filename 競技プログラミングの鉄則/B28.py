"""
N = int(input())
fib = [1,1]

for i in range(2,N):
    fib.append((fib[i-2]+fib[i-1])%(10**9+7))

print(fib[-1])
"""
# リストは遅い、変数だけでやってみる
N = int(input())
A,B = 1,1
waru = 10**9+7
for i in range(N-2):
    C = (A+B)%waru
    A,B = B,C

print(B)