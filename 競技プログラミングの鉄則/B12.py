N = int(input())

def f(x):
    return x**3 + x

left = 0
right = 100

for i in range(100):
    mid = (left + right) / 2
    y = f(mid)

    if y > N:
        right = mid
    else:
        left = mid

print(mid)