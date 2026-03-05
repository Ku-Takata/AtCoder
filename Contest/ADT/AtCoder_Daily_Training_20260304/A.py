S = list(input())

for i in range(0,len(S)-1,2):
    S[i],S[i+1] = S[i+1],S[i]

print("".join(S))