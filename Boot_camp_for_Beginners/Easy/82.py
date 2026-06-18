Q, H, S, D = map(int,input().split())
N = int(input())

# ちょうどNである必要がある
# 一旦どれが安いかを判定して、安い順に貪欲に買えば良い
# Nは整数だから1か2Lで考えれば良い

one = min(Q*4,H*2,S)
cheap = min(one*2,D)
expensive = max(one*2,D)

if cheap == one*2:
    ans = N*(cheap//2)
else:
    ans = (N//2)*cheap
    if N % 2 == 1:
        ans += expensive//2

print(ans)