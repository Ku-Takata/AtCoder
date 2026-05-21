N = int(input())
A = list(map(int,input().split()))

# 2つのペアが作れる最大の棒を2つ選ぶ

from collections import Counter
cnt_A = Counter(A).most_common()
cnt_A.sort(key=lambda x: x[0],reverse=True)

pair1 = 0
pair2 = 0

for i in range(len(cnt_A)):
    if cnt_A[i][1] < 2:
        continue
    elif pair1 == 0:
        pair1 = cnt_A[i][0]
        if cnt_A[i][1] >= 4:
            pair2 = cnt_A[i][0]
    elif pair2 == 0:
        pair2 = cnt_A[i][0]
    else:
        break

print(pair1*pair2)