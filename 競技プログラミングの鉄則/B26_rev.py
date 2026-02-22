# B26.pyだと既にFalseになっているのをFalseにしているのと、素数ではないものでループしているので無駄がある
# また、リストの保持の仕方が複雑でデバッグしにくい

N = int(input())
# インデックスを数として考える
prime = [True for i in range(N+1)]

for i in range(2,int(N**(1/2))+1):
    if not prime[i]:
        continue

    for j in range(i**2,N+1,i):
        prime[j] = False

for i in range(2,N+1):
    if prime[i]:
        print(i)

# あとは2以外の偶数を最初からFalseにしてリストを作ると計算量を削減できると思う。
# あとforループではなくスライスを使った一括代入