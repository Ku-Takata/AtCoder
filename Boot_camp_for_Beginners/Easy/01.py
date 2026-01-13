A, B = map(int, input().split())
i = 0

while True:
    if i == 1:
        if A*i >= B:
            print(i)
            exit()
    elif i >= 2:
        if A*i - (i-1) >= B:
            print(i)
            exit()
    elif i == 0:
        if B == 1:
            print(i)
            exit()
    
    i += 1