N = int(input())
S = list(input())
Q = int(input())

import string
alphabet = list(string.ascii_lowercase)

for i in range(Q):
    c, d = map(str,input().split())
    for j in range(26):
        if alphabet[j] == c:
            alphabet[j] = d

for i in range(N):
    idx = ord(S[i]) - ord("a")
    S[i] = alphabet[idx]

print("".join(S))