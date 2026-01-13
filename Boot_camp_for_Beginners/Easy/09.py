N = int(input())
K = int(input())
X = list(map(int, input().split()))
distance = []

for x in X:
    if abs(0 - x) < abs(K - x):
        distance.append(abs(0-x) * 2)
    else:
        distance.append(abs(K - x) * 2)

print(sum(distance))