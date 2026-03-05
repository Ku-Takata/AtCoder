S = list(input())

if len(set(S)) != len(S):
    print("No")
    exit()

flag1 = False
flag2 = False

for i in range(len(S)):
    if S[i].isupper():
        flag1 = True
    if S[i].islower():
        flag2 = True

if flag1 and flag2:
    print("Yes")
else:
    print("No")