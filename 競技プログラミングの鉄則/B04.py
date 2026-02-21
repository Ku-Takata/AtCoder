N = list(map(int, str(input())))

j = 0
ans = 0

for i in range(len(N)-1,-1,-1):
    if N[i] == 1:
        ans += 2**j
    j += 1

print(ans)