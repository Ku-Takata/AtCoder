N = int(input())
S = []
for i in range(N):
    S.append(input())

m = 0

for i in range(N):
    if len(S[i]) > m:
        m = len(S[i])

for i in range(N):
    if len(S[i]) != m:
        S[i] = "."*((m-len(S[i]))//2) + (S[i]) + "."*((m-len(S[i]))//2)

for i in range(N):
    print(S[i])