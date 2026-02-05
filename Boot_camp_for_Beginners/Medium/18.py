C = []
for i in range(3):
    C.append(list(map(int,input().split())))

a1 = 0
b1 = C[0][0] - a1
b2 = C[0][1] - a1
b3 = C[0][2] - a1
a2 = C[1][0] - b1
a3 = C[2][0] - b1

a_list = [a1,a2,a3]
b_list = [b1,b2,b3]

for i in range(3):
    for j in range(3):
        if C[i][j] != a_list[i]+b_list[j]:
            print("No")
            exit()

print("Yes")