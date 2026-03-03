S = list(map(int,input().split()))

for i in range(7):
    if S[i] > S[i+1]:
        print("No")
        exit()
    if S[i] % 25 != 0:
        print("No")
        exit()

if S[0] < 100 or S[7] > 675:
    print("No")
    exit()

print("Yes")