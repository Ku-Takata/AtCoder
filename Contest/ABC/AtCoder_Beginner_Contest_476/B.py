N = int(input())
S = input()
T = input()

for i in range(N):
    if T[i] != "*":
        if S[i] != T[i]:
            print("No")
            exit()
        else:
            continue

print("Yes")