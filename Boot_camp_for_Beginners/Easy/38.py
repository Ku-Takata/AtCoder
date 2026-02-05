order_list = []
for i in range(5):
    order_list.append(int(input()))

min_mod = 10
last_order = order_list[0]

for i in range(5):
    mod = order_list[i] % 10
    if mod < min_mod and mod != 0:
        min_mod = mod
        last_order = order_list[i]

order_list.remove(last_order)
total = 0

for i in range(4):
    if order_list[i] % 10 == 0:
        total += order_list[i] // 10
    else:
        total += order_list[i] // 10 + 1

print(total*10 + last_order)