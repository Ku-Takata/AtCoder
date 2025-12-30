H, W, N = map(int, input().split())
A = []
B = []
count = 0
count_list = []



for h in range(H):
    a = list(map(int, input().split()))
    A.append(a)

for n in range(N):
    b = int(input())
    B.append(b)

for i in range(len(A)):
    for b in B:
        if b in A[i]:
            count += 1
    count_list.append(count)
    count = 0

result = max(count_list)

print(result)