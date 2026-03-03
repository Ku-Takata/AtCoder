N = int(input())

for i in range(N+1):
    if i == 0 or i == 1:
        total = 1
    elif i < 6:
        total = 2**(i-1)
    else:
        temp1 = 0
        temp2 = total
        while temp2 > 0:
            temp1 += temp2 % 10
            temp2 = temp2 // 10
        total += temp1
print(total)