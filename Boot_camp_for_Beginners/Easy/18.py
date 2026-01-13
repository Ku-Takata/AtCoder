S = input()

count = 0
count_list = []

for s in S:
    if s == "A" or s == "C" or s == "G" or s == "T":
        count += 1
    else:
        count_list.append(count)
        count = 0
    count_list.append(count)

print(max(count_list))