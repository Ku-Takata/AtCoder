S = input()
T = input()

import string

alphabet = list(string.ascii_lowercase)
slide_list = []

for i in range(len(S)):
    slide = (alphabet.index(S[i]) - alphabet.index(T[i])) % len(alphabet)
    slide_list.append(slide)

if len(list(set(slide_list))) > 1:
    print("No")
else:
    print("Yes")