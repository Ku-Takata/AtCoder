S = input()

if S[0] != "A":
    print("WA")
    exit()
if S[2:-1].count("C") != 1:
    print("WA")
    exit()
if sum(1 for char in S if char.isupper()) != 2:
    print("WA")
    exit()

print("AC")