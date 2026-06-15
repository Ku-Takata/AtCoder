N = list(map(int, str(int(input()))))

# ある桁の値を−1するより、それ以降の値を9にしたときにプラスならその処理を行う

for i in range(len(N)-1):
    flag = False
    if N[i] > 0:
        for j in range(i+1,len(N)):
            if N[j] < 9:
                flag = True
            break

    if flag:
        N[i] -= 1
        for j in range(i+1,len(N)):
            N[j] = 9

# print(N)
print(sum(N))