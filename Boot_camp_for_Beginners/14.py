N,K = map(int, input().split())

# この愚直なやり方だとN, Kが大きいときに計算量が膨大になる
"""N_list = []

while N not in N_list:
    N_list.append(N)
    N = abs(N - K)

print(min(N_list))"""

# Kを繰り返し引く計算は割り算に当たる
# tを余りとすると、t < Kまで繰り返し、その後の計算は、|t - K| = K - t, |(K - t) - K| = t となる。つまり、tかK-tが最小となる
print(min(N % K, K - (N % K)))