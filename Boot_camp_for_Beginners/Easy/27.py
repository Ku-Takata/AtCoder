N,A,B = map(int,input().split())

temp = divmod(N,(A+B))

blue = A * temp[0]
if temp[1] >= A:
    blue_amari = A
else:
    blue_amari = temp[1]

print(blue+blue_amari)

# 7