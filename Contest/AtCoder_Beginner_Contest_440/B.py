N = int(input())
T = list(map(int,input().split()))
sorted_T = sorted(T)
index_list = []

for i in range(3):
    index_list.append(T.index(sorted_T[i])+1)

print(*index_list)