N = int(input())
X = list(map(int,input().split()))

# Xの要素数は必ず偶数であり、1つ除くと必ず奇数になるため、中央値は必ず1つの要素で決まる
# 毎回1つしか取り除かないなら、中央値の変化は2つか3つの要素しか見なくて済みそう

sort_X = sorted(X)
median = sort_X[(N//2)-1]
median_after = sort_X[N//2]

for i in X:
    if i <= median:
        print(median_after)
    elif i > median:
        print(median)