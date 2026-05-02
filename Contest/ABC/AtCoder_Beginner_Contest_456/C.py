S = input()

# 隣合わない所まで部分文字列をまとめて、隣り合ったら一旦そこまでを切って、隣り合わない組み合わせを計算かな

renzoku = 1
ans = 0

for i in range(len(S)-1):
    if S[i] != S[i+1]:
        renzoku += 1
    else:
        ans += (renzoku*(renzoku+1))//2
        renzoku = 1

ans += (renzoku*(renzoku+1))//2

print(ans % 998244353)