N, X = map(str,input().split())
N = int(N)
alphabet = ["A","B","C","D","E"]
for i in range(5):
    if alphabet[i] == X:
        X = i
S = [list(input()) for i in range(N)]

for i in range(N):
    if S[i][X] == "o":
        print("Yes")
        exit()

print("No")