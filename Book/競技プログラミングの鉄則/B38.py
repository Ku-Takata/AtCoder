N = int(input())
S = input()

grass = [1]*N

for i in range(N-1):
    if S[i] == "A":
        grass[i+1] = grass[i] + 1

for i in range(N-2,-1,-1):
    if S[i] == "B":
        grass[i] = max(grass[i+1]+1,grass[i])

# print(grass)
print(sum(grass))