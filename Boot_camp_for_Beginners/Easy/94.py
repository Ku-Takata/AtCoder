ABCD = list(input())
for i in range(4):
    ABCD[i] = int(ABCD[i])

from itertools import product


for p in product(["+","-"],repeat=3):
    ans = ABCD[0]
    for i in range(3):
        if p[i] == "+":
            ans += ABCD[i+1]
        else:
            ans -= ABCD[i+1]

    if ans == 7:
        print(f"{ABCD[0]}{p[0]}{ABCD[1]}{p[1]}{ABCD[2]}{p[2]}{ABCD[3]}=7")
        exit()