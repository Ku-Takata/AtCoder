Q = int(input())

from collections import deque
import heapq
snake = deque()

temp = 0

for i in range(Q):
    query = list(map(int,input().split()))

    if query[0] == 1:
        if len(snake) > 0:
            snake.append(snake[-1]+length)
            length = query[1]
        else:
            snake.append(0)
            length = query[1]
    elif query[0] == 2:
        snake.popleft()
    else:
        print(snake[query[1]-1] - snake[0])