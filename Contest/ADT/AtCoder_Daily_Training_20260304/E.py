S = input()

# アルファベットは26個
import string
alphabet = list(string.ascii_uppercase)

S_rev = []

for char in reversed(S):
    S_rev.append(alphabet.index(char))

# print(S_rev)
ans = 0

for i in range(len(S_rev)):
    ans += (S_rev[i]+1) * (26**i)

print(ans)