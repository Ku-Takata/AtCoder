H, W = map(int,input().split())
S = [list(input()) for i in range(H)]

# 移動回数制限が5*10**6
# それ以下の移動方法を1つ出せばよい
# これもDFSで解けそうだけど、戻るのが正解だったりするから難しそう
# 戻るのが正解のケースはどんな時か考える
# xとoの時、任意のマスに移動できないので一旦別の角度から踏む必要が出る場面がある
# xとoを踏んだ時だけ戻るのを許しても良いようにする
# ただ何回も戻っていいわけではないので、もし戻って、今まで行ったことのないところに行けない場合はreturnで良さそう
# xとoの時のDFS難しすぎる問題
# なんかポケモンにこんなのあった気がする。ダイパの雪のジムみたいな。

dx = [1,-1,0,0]
dy = [0,0,1,-1]

for i in range(H):
    if "S" in S[i]:
        start = [i,S[i].index("S")]
    if "G" in S[i]:
        goal = [i,S[i].index("G")]

visited = [[False]*W for i in range(H)]

def dfs(x,y,path):
    if [x,y] == goal:
        return path
    if visited[x][y] == True:
        return

    for i in range(4):
        nx = x + dx[i]
        ny = y + dy[i]

        if 0 <= nx < W and 0 <= ny < H:
            dfs(nx,ny)

ans = []
dfs(start[1],start[0],ans)