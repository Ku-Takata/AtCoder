N = int(input())
C = list(map(int,input().split()))

from collections import Counter

cnt_C = Counter(C)
most_common = cnt_C.most_common(1)[0]
print(N - most_common[1])