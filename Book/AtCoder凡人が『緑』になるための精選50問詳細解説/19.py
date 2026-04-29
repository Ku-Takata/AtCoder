X,K,D = map(int,input().split())

# Kは10^15あるので、全探索は不可。初めて一番小さい値になる回数を探す。
if X != 0:
    cnt_zero_temae = X//D
    cnt_nokori = K - cnt_zero_temae
else:
    cnt_zero_temae = 0
    cnt_nokori = K

if abs(K*D) > abs(X):
    if cnt_nokori % 2 == 0:
        ans = abs(X % D)
    else:
        ans = abs((X % D) - D)
else:
    ans = abs(X) - K*D

print(ans)

# 30 普通に解けたが、少し時間がかかった。