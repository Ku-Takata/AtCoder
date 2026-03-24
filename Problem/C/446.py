from collections import deque
import heapq

T = int(input())
for _ in range(T):
    N, D = map(int,input().split())
    A = list(map(int,input().split()))
    B = list(map(int,input().split()))

# Aの長さがD個以上の長さになったら先頭を捨てる
# もしくは卵の個数が0になったら捨てる

    egg = deque()
    for i in range(N):
        egg.append(A[i])
        if egg[0] - B[i] > 0:
            egg[0] -= B[i]
        elif egg[0] - B[i] == 0:
            egg.popleft()
        else:
            while B[i] > 0:
                if egg[0] - B[i] >= 0:
                    egg[0] -= B[i]
                    break
                B[i] -= egg[0]
                egg[0] = 0
                egg.popleft()

        if len(egg) > D:
            egg.popleft()

        # print(egg)
    print(sum(egg))

# pop(0)は計算量O(1)かと思いきやO(N)かかる
# 普通にdequeを使おう