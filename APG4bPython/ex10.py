N = int(input())
A = int(input())

for i in range(N):
    op, B = input().split()
    B = int(B)
    if op == "+":
        A += B
    elif op == "-":
        A -= B
    elif op == "*":
        A *= B
    elif op == "/" and B != 0:
        A //= B
    else:
        print("error")
        break
    print(i + 1, A)