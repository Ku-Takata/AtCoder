N, K = map(int,input().split())
H = list(map(int,input().split()))

# TLE!ﾊﾟﾜｰ!
"""
max_H = max(H[:K])
min_H = min(H[:K])
hendou = max_H - min_H
K_window = H[:K]
flag_max = False
flag_min = False
flag_hendou = False

for i in range(1,N-K+1):
    if max_H == K_window[0] or max_H < H[K+i-1]:
        flag_max = True
        flag_hendou = True
    if min_H == K_window[0] or min_H > H[K+i-1]:
        flag_min = True
        flag_hendou = True

    K_window.pop(0)
    K_window.append(H[K+i-1])

    if flag_max:
        max_H = max(K_window)
        flag_max = False
    if flag_min:
        min_H = min(K_window)
        flag_min = False

    if flag_hendou:
        if hendou < max_H - min_H:
            hendou = max_H - min_H
        flag_hendou = False

print(hendou)
"""

# デキューでやってみよう


