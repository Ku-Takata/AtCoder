N, M = map(int, input().split())
S = input()
T = input()

count_list = []

S_list = []
T_list = []

# S, Tの文字列から1文字ずつ要素としてリストに格納する
for s in S:
    S_list.append(s)

for t in T:
    T_list.append(t)

# 探索する回数はN-M+1
for i in range(N-M+1):
    count = 0
    S_list_slice = S_list[i:i+M]
    # それぞれの探索場所で一致しているかしていないかを判断し、しているなら0を、していないなら計算を行う
    if T_list != S_list_slice[i:i+M]:
        for j in range(len(T)):
            if T_list[j] > S_list_slice[j]:
                count += 10 - int(T_list[j]) + int(S_list_slice[j])
                # print(10 - int(T_list[j]) + int(S_list_slice[j]))
            elif T_list[j] <= S_list_slice[j]:
                count += int(S_list_slice[j]) - int(T_list[j])
                # print(int(S_list_slice[j]) - int(T_list[j]))
        count_list.append(count)
    if T_list == S_list_slice[i:i+M]:
        count_list.append(0)

# print(count_list)
print(min(count_list))