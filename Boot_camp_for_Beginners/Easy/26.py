H = int(input())

cnt = 1

# 何回半分にできるか問題
while H != 1:
    H = H // 2
    cnt += 1

ans = 2**cnt - 1
print(ans)

# 6