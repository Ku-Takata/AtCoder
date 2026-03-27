S = input()

for i in range(len(S)):
    if S[i] == "A":
        start = i
        break

for i in range(len(S)-1,-1,-1):
    if S[i] == "Z":
        end = i
        break

print(end-start+1)