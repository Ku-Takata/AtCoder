N = int(input())
prime = [i for i in range(2,N+1)]

# エラトステネスのふるい
for i in range(2,int(N**(1/2))+1):
    # print(2*i-2)
    for j in range(2*i-2,N-1,i):
        prime[j] = False
        # print(prime)

for i in range(len(prime)):
    if prime[i]:
        print(prime[i])
    else:
        None