N = int(input())

from collections import deque
import heapq
N_list = deque()
N_list.append(N)

total = 0

while len(N_list) > 0:
    if N_list[0] == 1:
        N_list.popleft()
    elif N_list[0] % 2 == 0:
        calc = N_list[0] // 2
        total += N_list.popleft()
        N_list.append(calc)
        N_list.append(calc)
    else:
        calc1 = N_list[0] // 2
        calc2 = N_list[0] // 2 + 1
        total += N_list.popleft()
        N_list.append(calc1)
        N_list.append(calc2)

    # print(N_list)

print(total)

# これだとTLEになる。やっぱりDFSかな。