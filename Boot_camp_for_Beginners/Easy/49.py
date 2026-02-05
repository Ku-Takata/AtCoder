H,W = map(int,input().split())

for i in range(H+2):
    if i == 0 or i == H+1:
        print("#"*(W+2))
    else:
        print("#"+input()+"#")