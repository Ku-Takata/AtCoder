N = int(input())

# 全列挙してソートするので計算量間に合わないのかな
# サンプルケースを見ると10**6くらいのリストっぽい
# 2のべき乗を全列挙して、それから小さい値を順にくっつける作業を行うBFS的なもの？

two_pow = []
for i in range(10**6):
    if 2**i <= 10**9:
        two_pow.append(2**i)
    else:
        break
# print(two_pow)

import heapq
que = []
visited = set()

for i in two_pow:
    heapq.heappush(que, i)
    visited.add(i)

# print(visited)
cnt = 0

while que:
    current = heapq.heappop(que)
    cnt += 1

    if cnt == N:
        print(current)
        break

    for i in two_pow:
        new_num = int(str(current) + str(i))

        if new_num > 10**9:
            break

        if new_num not in visited:
            visited.add(new_num)
            heapq.heappush(que, new_num)