N = list(input())

for i in range(len(N)):
    if N[0] != N[i]:
        print("No")
        exit()

print("Yes")