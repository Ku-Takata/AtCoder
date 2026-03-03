P,Q = map(int,input().split())
X,Y = map(int,input().split())

if P <= X < P+100 and Q <= Y < Q+100:
    print("Yes")
else:
    print("No")