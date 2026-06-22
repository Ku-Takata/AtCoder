ABC = list(map(int,input().split()))

# まず2つを同じ状態にした後に、1つを同じ値にしたい
# 奇数を一番大きくしたい

ABC.sort()
total = (ABC[2]-ABC[1]) + (ABC[2]-ABC[0])

if sum(ABC) % 2 == (3*ABC[2]) % 2:
    print(total//2)
else:
    print((total+3)//2)

# 個人的にはムズイ。偶奇が同じもの同士を引くと答えが偶数になり、違うと奇数になるという特性から解法を思いつかないといけない。