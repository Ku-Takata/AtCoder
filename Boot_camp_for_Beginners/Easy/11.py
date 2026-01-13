N = int(input())
count = 0
count_list = []

for n in range(N+1):
    if n == 0:
        count = 0
        continue
    while n % 2 == 0:
        n //= 2
        count += 1
    count_list.append(count)
    count = 0

print(count_list.index(max(count_list)) + 1)