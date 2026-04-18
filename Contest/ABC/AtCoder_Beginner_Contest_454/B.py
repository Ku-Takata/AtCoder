N, M = map(int,input().split())
F = list(map(int,input().split()))

if len(set(F)) == len(F):
    print("Yes")
else:
    print("No")

if len(set(F)) == M:
    print("Yes")
else:
    print("No")