N = list(str(input()))
N_list = [int(n) for n in N]

# 各桁を足し合わせて3の倍数になるかを見たら良い。
# 消すときに割ったあまりが1か2かで3の倍数にできるかどうか判定。
total = sum(N_list)
cnt_1 = 0
cnt_2 = 0

for n in N_list:
    if n % 3 == 1:
        cnt_1 += 1
    elif n % 3 == 2:
        cnt_2 += 1
    else:
        None

if total % 3 == 0:
    print(0)
elif total % 3 == 2:
    if cnt_2 >= 1 and len(N_list) > 1:
        print(1)
    elif cnt_1 >= 2 and len(N_list) > 2:
        print(2)
    else:
        print(-1)
elif total % 3 == 1:
    if cnt_2 >= 2 and len(N_list) > 2:
        print(2)
    elif cnt_1 >= 1 and len(N_list ) > 1:
        print(1)
    else:
        print(-1)
