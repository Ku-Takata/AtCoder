"""N = int(input())
N_list = list(range(1, N+1))
k = 0
A = []
x = 0
y = 0
x_list = []
y_list = []

# x^2 + y^2 <= n, 0 < x < yを満たすx,yの組の数とその時のnを求める
# y^2 < nまでy+1を繰り返し、その中からy^2 + x^2 = nになる組み合わせが1つだけのものをカウント、リストアップしたらOK

for n in N_list:
    while y**2 < n:
        y_list.append(y+1)
        x_list.append(x+1)
        y += 1

    for y in y_list:
        for i in range(len(y_list)):
            if y**2 + x_list[i] == n:
                count += 1


            if count == 2:
                break

# やろうとしたことを仮に実装できたとしても、O(N√N)となり、N <= 10^7まであることからTLEになってしまう"""


# Z_atcoderさんのコードを見てどうやったらよかったのかを学ぶ
N = int(input())
v = [0]*(N+1)

for x in range(1,N):
    # x^2がNをオーバーするならループ終了
    if x**2 > N:
        break
    # x < yであるからx+1から始める。最大がN+1としているのはy=Nまで行くことは無いが、とりあえず取りえる値までループできるようにしておいただけ。
    # 厳密に求めても、以下の条件式で取りえる値までしか回らないので、計算量や出力に変化は無い
    for y in range(x+1,N+1):
        # Nを越えたらループ終了
        if x**2 + y**2 > N:
            break
        # 計算結果のインデックスの値を+1する。これによって、すべての計算結果の値が何回作れたかをリストに保存できる。
        v[x**2 + y**2] += 1

result = []

for x in range(1, N+1):
    # 良い数をリストに順番に格納
    if v[x] == 1:
        result.append(x)

print(len(result))
print(*result)