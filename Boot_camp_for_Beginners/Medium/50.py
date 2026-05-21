L, R = map(int,input().split())

# 2019からどれだけ値が離れているかを問う問題
# Lを2019で割って余りを出す
# そこからR-L個数があるわけだから、その個数分足すうちに2019の倍数があるか確認で良い

mod_L = L % 2019
cnt = R - L

if mod_L + cnt >= 2019:
    print(0)
else:
    ans = float("inf")
    for i in range(mod_L, mod_L+cnt+1):
        for j in range(i+1,mod_L+cnt+1):
            ans = min((i*j)%2019, ans)

    print(ans)