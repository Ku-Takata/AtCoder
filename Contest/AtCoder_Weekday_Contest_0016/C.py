N, L, R, T = map(int,input().split())

price = float("inf")
taste = float("-inf")
ans = -1

for i in range(N):
    P, S = map(int,input().split())

    # まず値段とおいしさを満たす条件、おいしさよりも値段の方を重要視
    if L <= P <= R and T <= S:
        if price == P:
            if taste == S:
                continue
            elif taste < S:
                price = P
                taste = S
                ans = i+1
        elif P < price:
            price = P
            taste = S
            ans = i+1

print(ans)