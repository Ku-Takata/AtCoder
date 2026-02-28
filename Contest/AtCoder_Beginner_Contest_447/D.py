S = input()

cnt_A = 0
cnt_AB = 0
cnt_ABC = 0

for i in range(len(S)):
    if S[i] == "A":
        cnt_A += 1
    elif S[i] == "B" and cnt_A > 0:
        cnt_A -= 1
        cnt_AB += 1
    elif S[i] == "C" and cnt_AB > 0:
        cnt_ABC += 1
        cnt_AB -= 1

print(cnt_ABC)