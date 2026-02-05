N = int(input())
Lucas_num = [2,1]

for i in range(2,N+1):
    Lucas_num.append(Lucas_num[i-1]+Lucas_num[i-2])

print(Lucas_num[N])