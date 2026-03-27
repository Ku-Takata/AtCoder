O = input()
E = input()

ans = [0]*(len(O)+len(E))

for i in range(len(O)):
    ans[2*(i+1)-2] = O[i]

for i in range(len(E)):
    ans[2*(i+1)-1] = E[i]

print("".join(ans))