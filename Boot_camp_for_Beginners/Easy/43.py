w = list(input())
w_kinds = list(set(w))

for i in range(len(w_kinds)):
    if w.count(w_kinds[i]) % 2 != 0:
        print("No")
        exit()

print("Yes")