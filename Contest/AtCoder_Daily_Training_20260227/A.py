S = input()

for i in range(1,len(S),2):
    if S[i] == "1":
        print("No")
        exit()

print("Yes")