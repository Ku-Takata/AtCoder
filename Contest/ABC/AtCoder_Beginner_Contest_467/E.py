N, M = map(int,input().split())
A = list(map(int,input().split()))
B = list(map(int,input().split()))

# M = 2だけ
# 別にM自由でも簡単そうだけど
# この場合偶奇だけ考える
# E挑戦してみるか
# AiとAi+1を足し合わせて、Mで割ったときの余りがBiより大きいか小さいかチェック
# 小さかったら+1を余りとの差分足し合わせ、カウント
# 大きかったら
# Eのサンプルは何か逆順で+1やってる
# ちょっと後でEはやろう
# WA??
# 最初のA0を足した方が短いケースがあるってことか

cnt1 = 0
A_copy = A.copy()
for i in range(N-1):
    A_mod = (A[i]+A[i+1]) % M
    if A_mod != B[i]:
        A[i+1] += 1
        cnt1 += 1

A_copy[0] += 1
cnt2 = 1
for i in range(N-1):
    A_mod = (A_copy[i]+A_copy[i+1]) % M
    if A_mod != B[i]:
        A_copy[i+1] += 1
        cnt2 += 1

print(min(cnt1,cnt2))