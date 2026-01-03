N = input()
happy = []
total = 0
i = 0

"""while N not in N_list and N != 1:
    N_list.append(N)

    for n in N:
        happy.append(int(n))

    N = 0

    for x in happy:
        N += x**2
    happy.clear()
    happy.append(N)
    print(N, happy, N_list)

happy.append(N)"""

while len(set(happy)) == len(happy):
    if i == 0:
        N_list = [int(x) for x in list(str(N))]
        i += 1
    else:
        N_list = [int(x) for x in list(str(N_list[0]))]

    total = 0

    for n in N_list:
        total += n ** 2

    N_list.clear()
    N_list.append(total)
    happy.append(total)
    # print(total, N_list, happy)


if 1 in happy:
    print("Yes")
else:
    print("No")