A,B = map(int,input().split())
S = input()

for i in range(len(S)):
    if i == A:
        if S[i] == "-":
            continue
        else:
            print("No")
            exit()
    else:
        if S[i] != "-":
            continue
        else:
            print("No")
            exit()

print("Yes")