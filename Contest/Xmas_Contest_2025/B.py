import itertools

N = int(input())
fibonacci = []

for i in range(N+2):
    if i == 0:
        fibonacci.append(1)
    elif i == 1:
        fibonacci.append(1)
    else:
        fibonacci.append(fibonacci[i-1] + fibonacci[i-2])

m = fibonacci[N+1]

"""print(m)"""

gyouretu = list(itertools.product([0, 1], repeat=N))

"""print(gyouretu)"""

tyouhukunashi = []

tyouhukunashi.append(gyouretu[0])

for i in range(1, m):
    if gyouretu[i] != gyouretu[i-1]:
        tyouhukunashi.append(gyouretu[i])

"""print(tyouhukunashi)"""

tyouhukunashi = str(tyouhukunashi)

for t in range(len(tyouhukunashi)):
    print(''.join(tyouhukunashi[t]))