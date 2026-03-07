A = list(map(int, str(int(input()))))
ans = list(map(int, str(int(input()))))

marubatsu = []

for i in range(len(A)):
    if A[i] == ans[i]:
        marubatsu.append(f"{i+1}: o")
    else:
        marubatsu.append(f"{i+1}: x")

print(marubatsu)
cnt = 0

for i in range(len(A)):
    if marubatsu[i] == f"{i+1}: o":
        cnt += 1

print(cnt, cnt/len(A))