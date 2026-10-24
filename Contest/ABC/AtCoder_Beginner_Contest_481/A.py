A, B = map(int,input().split())

if A + B >= 100:
    print(3)
elif A + B >= 10:
    print(2)
else:
    print(1)