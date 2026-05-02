S = input()

dp = {"a":0, "b":0, "c":0}
waru = 998244353

for char in S:
    dp[char] = (dp['a'] + dp['b'] + dp['c'] + 1) % waru

print(sum(dp.values()) % waru)