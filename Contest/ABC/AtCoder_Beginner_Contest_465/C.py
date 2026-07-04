from collections import deque

N = int(input())
S = input()

# reverseの印を付けて先頭から入れるか、後ろから入れるか判定する

A = deque()
reverse = False

for i in range(1,N+1):
    if reverse:
        A.appendleft(i)
    else:
        A.append(i)

    if S[i-1] == "o":
        reverse = not reverse

if reverse:
    A.reverse()

print(*A)