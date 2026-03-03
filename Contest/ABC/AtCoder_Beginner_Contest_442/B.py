Q = int(input())

volume = 0
play = 0

for i in range(Q):
    A = int(input())

    if A == 1:
        volume += 1
    elif A == 2 and volume >= 1:
        volume -= 1
    elif A == 3 and play == 0:
        play = 1
    elif A == 3 and play == 1:
        play = 0

    if volume >= 3 and play == 1:
        print("Yes")
    else:
        print("No")

# 特に躓いたりはしなかった。