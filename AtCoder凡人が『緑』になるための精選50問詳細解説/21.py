A,B,W = map(int,input().split())

W *= 1000

for i in range(1,10**8):
    if A*i <= W <= B*i:
        min_num = i
        break
    if A*i > W:
        print("UNSATISFIABLE")
        exit()

max_num = 0

for i in range(1,10**8):
    if A*i <= W <= B*i:
        max_num = i
        continue
    if A*i > W:
        if max_num == 0:
            print("UNSATISFIABLE")
            exit()
        else:
            break

print(min_num, max_num)

# 27 解法を思いつくのに時間がかかった。実装自体は難しくない。
# 今回は最大と最小を分けてループしたが、同時にもできる。解説のコードはループを回し切って、最小と最大を求めている。