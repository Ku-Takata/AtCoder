X = list(input())
N = int(input())
S = [input() for i in range(N)]

custom_dict = {char: i for i,char in enumerate(X)}

sort_S = sorted(S, key=lambda word: [custom_dict[i] for i in word])

for i in range(N):
    print(sort_S[i])