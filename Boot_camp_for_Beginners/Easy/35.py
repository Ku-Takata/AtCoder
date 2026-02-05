S = set(list(input()))

import string
alpha_list = list(string.ascii_lowercase)

for i in range(len(alpha_list)):
    if alpha_list[i] not in S:
        print(alpha_list[i])
        exit()

print("None")