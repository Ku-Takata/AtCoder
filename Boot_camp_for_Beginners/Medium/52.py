N = int(input())
task = [list(map(int,input().split())) for i in range(N)]

# 鉄則本で見た事あるような問題、おそらく貪欲法
# 普通に考えたら、時間が迫っているものからやったら良い

task.sort(key=lambda x:x[1])
time = 0

for i in range(N):
    time += task[i][0]
    if time > task[i][1]:
        print("No")
        exit()

print("Yes")