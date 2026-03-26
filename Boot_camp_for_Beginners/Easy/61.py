S = list(input())
T = list(input())

from collections import deque
import heapq
rotate = deque(S)
fix = deque(T)

for i in range(len(S)):
    rotate.appendleft(rotate.pop())
    if rotate == fix:
        print("Yes")
        exit()

print("No")