data = [int(c) for c in input().split()]
# dataの中で隣り合う等しい要素が存在するなら"YES"を出力し、そうでなければ"NO"を出力する
# ここにプログラムを追記
count = 0

for i in range(len(data)-1):
    if data[i+1] == data[i]:
        print("YES")
        break
    else:
        count += 1
        if count == len(data)-1:
            print("NO")