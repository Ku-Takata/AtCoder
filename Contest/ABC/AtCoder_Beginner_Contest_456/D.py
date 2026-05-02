S = input()

# 次は部分文字列ではなく、部分列
# 普通にDPで行けそう
# それまでのa,b,cの数をカウントして、できる組み合わせを計算

dp = {"a":0, "b":0, "c":0}

# abbcなら、aでa:1,b:0,c:0、b,abでa:1,b:2,c:0、b,abでa:1,b:2+2,c:0、c,ac,bc,bc,abc,abcでa:1,b:4,c:6
# 合計1+4+6=11
# なんかTLEになったな
# 常に余りで計算しないと桁が大きくなってるってことか？
# 鉄則本にあるし多分そう

waru = 998244353

for char in S:
    if char == "a":
        cnt = (dp["b"] + dp["c"] + 1) % waru
        dp["a"] = dp["a"] + cnt
    elif char == "b":
        cnt = (dp["a"] + dp["c"] + 1) % waru
        dp["b"] = dp["b"] + cnt
    elif char == "c":
        cnt = (dp["a"] + dp["b"] + 1) % waru
        dp["c"] = dp["c"] + cnt

print(sum(dp.values()) % waru)