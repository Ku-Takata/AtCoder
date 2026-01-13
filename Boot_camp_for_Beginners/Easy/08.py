a, b = input().split()

ab = int(a+b)
nizyoukon = ab ** (1/2)

if nizyoukon.is_integer():
    print("Yes")
else:
    print("No")