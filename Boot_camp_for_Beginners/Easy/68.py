N = int(input())
restaurant = [[i+1,S,int(P)] for i in range(N) for S,P in [input().split()]]

restaurant.sort(key=lambda x: (x[1],-int(x[2])))
# print(restaurant)

for i in range(N):
    print(restaurant[i][0])