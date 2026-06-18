Q, H, S, D = map(int, input().split())
N = int(input())

# ちょうどNである必要がある
# 一旦どれが安いかを判定して、安い順に貪欲に買えば良い
# Nは整数だから1か2Lで考えれば良い

# 1L最安値
one = min(Q*4, H*2, S)
# 2Lでの最安値
cheap = min(one*2, D)
# まずは2L単位で買えるだけ一番安い方で買う
ans = (N//2) * cheap
# もしNが奇数なら、1Lの最安値を1個だけ足す
if N % 2 == 1:
    ans += one

print(ans)