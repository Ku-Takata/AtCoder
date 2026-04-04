H, W = map(int,input().split())

print("#"*W)

for i in range(H-2):
    print("#"+"."*(W-2)+"#")

print("#"*W)