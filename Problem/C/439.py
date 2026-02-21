N = int(input())

N_double = []
i = 1

while i**2 < 10**7:
    N_double.append(i**2)
    i += 1

A = []

for i in range(len(N_double)-1):
    x = N_double[i]
    if x  > N:
        break
    for j in range(i+1,len(N_double)):
        y = N_double[j]

        if x+y <= N and x+y not in A:
            A.append(x+y)
        else:
            break

A = sorted(A)

print(len(A))
print(*A)