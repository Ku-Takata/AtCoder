N = int(input())

import math

ans = bin(N)[2:]

while len(ans) != 10:
    ans = "0" + str(ans)

print(ans)