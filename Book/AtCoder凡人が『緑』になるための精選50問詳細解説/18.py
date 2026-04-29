N = int(input())
A = list(map(int,input().split()))

# A[i]*sum(A[i +1:]) + A[i+1]*sum(A[i+1 +1:]) ... というようになっている。
# sumだとTLEになった。全要素毎回見るから考えたら当然だった。
# Aの総和からA[i]を1ループごとに引いて、毎回sumで計算しなくても良いようにする。
ans = 0
wa = sum(A)

for i in range(N):
    ans += A[i]*(wa-A[i])
    wa = wa-A[i]

print(ans % (10**9 + 7))

# 15 式変形をしたら簡単なことに気づけたら簡単。数列の和系はおそらくこういう問題が良く出てくる。