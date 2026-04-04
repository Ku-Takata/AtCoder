S = input()
T = input()

from collections import deque
import heapq
S_que = deque()
T_que = deque(T)
cnt = 0

for i in range(len(S)):
    if i < len(T)-1:
        S_que.append(S[i])
        continue
    elif i == len(T)-1:
        S_que.append(S[i])
        if S_que == T_que:
            cnt += 1
        continue

    S_que.popleft()
    S_que.append(S[i])
    if S_que == T_que:
            cnt += 1

print(cnt)
print(len(S))