N, A, B = map(int, input().split())
S = input()
count = 0
count2 = 1

for i in range(len(S)):
    if S[i] == "a" and count < A+B:
        print("Yes")
        count += 1
    elif S[i] == "b" and count < A+B and count2 <= B:
        print("Yes")
        count += 1
        count2 += 1
    else:
        print("No")