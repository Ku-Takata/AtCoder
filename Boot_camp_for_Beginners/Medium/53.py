N = int(input())
A = list(map(int,input().split()))

# 最初に全部行く場合を考えて、その後に行かないスポットの前後から計算できそう

A_rev = [0] + A + [0]

pos = 0
total_all = 0
for a in A_rev:
    total_all += abs(pos-a)
    pos = a

for i in range(1,N+1):
    ans = total_all - abs(A_rev[i]-A_rev[i-1]) - abs(A_rev[i]-A_rev[i+1]) + abs(A_rev[i-1]-A_rev[i+1])

    print(ans)