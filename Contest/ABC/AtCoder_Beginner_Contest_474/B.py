N = int(input())
P = list(map(int,input().split()))

for i in range(int(N/10)+1):
    temp = (i+1)*10
    num = []
    for j in range(10):
        num.append(temp)
        temp -= 1

    temp = (i+1)*10

    # print(num)

    if N - temp >= 0:
        for j in range(temp-10,temp):
            # print(P[j])
            if P[j] not in num:
                # print(P[j])
                print("No")
                exit()
    else:
        for j in range(temp-10,N):
            if P[j] not in num:
                print("No")
                exit()

print("Yes")