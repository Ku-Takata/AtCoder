#include <bits/stdc++.h>
using namespace std;

static constexpr int INF = 1e9;

struct Pos{
    int r,c;
};

struct State{
    int r,c,d;
};

class Solver{
public:

    int N,M,T;

    vector<string> V;
    vector<string> H;

    vector<Pos> balls;
    vector<Pos> baskets;

    static constexpr int DR[4]={0,1,0,-1};
    static constexpr int DC[4]={1,0,-1,0};

    int S;

    vector<vector<unsigned short>> dists;
    vector<vector<int>> parent;
    vector<vector<char>> parent_move;

    Solver(){
        read_input();
    }

    inline int encode(
        int r,
        int c,
        int d
    ) const{
        return ((r*N+c)<<2)|d;
    }

    inline int row(
        int id
    ) const{
        return (id>>2)/N;
    }

    inline int col(
        int id
    ) const{
        return (id>>2)%N;
    }

    inline int dir(
        int id
    ) const{
        return id&3;
    }

    bool can_move(
        int r,
        int c,
        int d
    ) const{

        if(d==0){
            if(c+1>=N) return false;
            return V[r][c]=='0';
        }

        if(d==1){
            if(r+1>=N) return false;
            return H[r][c]=='0';
        }

        if(d==2){
            if(c-1<0) return false;
            return V[r][c-1]=='0';
        }

        if(d==3){
            if(r-1<0) return false;
            return H[r-1][c]=='0';
        }

        return false;
    }

    void read_input(){

        cin>>N>>M>>T;

        V.resize(N);

        for(int i=0;i<N;i++){
            cin>>V[i];
        }

        H.resize(N-1);

        for(int i=0;i<N-1;i++){
            cin>>H[i];
        }

        balls.resize(M);
        baskets.resize(M);

        for(int i=0;i<M;i++){

            cin
                >>balls[i].r
                >>balls[i].c
                >>baskets[i].r
                >>baskets[i].c;
        }
    }

    void build_apsp(){

        S=N*N*4;

        dists.assign(
            S,
            vector<unsigned short>(
                S,
                65535
            )
        );

        parent.assign(
            S,
            vector<int>(
                S,
                -1
            )
        );

        parent_move.assign(
            S,
            vector<char>(
                S,
                0
            )
        );

        vector<unsigned short> dist(S);

        for(int sr=0;sr<N;sr++){
            for(int sc=0;sc<N;sc++){
                for(int sd=0;sd<4;sd++){

                    int start=
                        encode(
                            sr,
                            sc,
                            sd
                        );

                    fill(
                        dist.begin(),
                        dist.end(),
                        65535
                    );

                    queue<int> q;

                    q.push(start);

                    dist[start]=0;

                    while(!q.empty()){

                        int cur=q.front();
                        q.pop();

                        int r=row(cur);
                        int c=col(cur);
                        int d=dir(cur);

                        unsigned short cd=
                            dist[cur];

                        if(can_move(r,c,d)){

                            int nr=
                                r+DR[d];

                            int nc=
                                c+DC[d];

                            int nxt=
                                encode(
                                    nr,
                                    nc,
                                    d
                                );

                            if(dist[nxt]
                               ==65535){

                                dist[nxt]=
                                    cd+1;

                                parent[start][nxt]
                                    =cur;

                                parent_move[start][nxt]
                                    ='F';

                                q.push(nxt);
                            }
                        }

                        {
                            int nd=
                                (d+1)&3;

                            int nxt=
                                encode(
                                    r,
                                    c,
                                    nd
                                );

                            if(dist[nxt]
                               ==65535){

                                dist[nxt]=
                                    cd+1;

                                parent[start][nxt]
                                    =cur;

                                parent_move[start][nxt]
                                    ='R';

                                q.push(nxt);
                            }
                        }

                        {
                            int nd=
                                (d+3)&3;

                            int nxt=
                                encode(
                                    r,
                                    c,
                                    nd
                                );

                            if(dist[nxt]
                               ==65535){

                                dist[nxt]=
                                    cd+1;

                                parent[start][nxt]
                                    =cur;

                                parent_move[start][nxt]
                                    ='L';

                                q.push(nxt);
                            }
                        }
                    }

                    for(int t=0;t<S;t++){

                        dists[start][t]
                            =dist[t];
                    }
                }
            }
        }
    }

    string restore_path(
        int sr,
        int sc,
        int sd,
        int tr,
        int tc,
        int &best_dir
    ){

        int start=
            encode(
                sr,
                sc,
                sd
            );

        int goal=-1;

        int best_cost=INF;

        for(int d=0;d<4;d++){

            int g=
                encode(
                    tr,
                    tc,
                    d
                );

            int cost=
                dists[start][g];

            if(cost<best_cost){

                best_cost=cost;
                goal=g;
                best_dir=d;
            }
        }

        string res;

        int cur=goal;

        while(cur!=start){

            res.push_back(
                parent_move[start][cur]
            );

            cur=
                parent[start][cur];
        }

        reverse(
            res.begin(),
            res.end()
        );

        return res;
    }

        mt19937 rng{
        (uint32_t)chrono::steady_clock::now()
        .time_since_epoch()
        .count()
    };

    int evaluate(
        const vector<int>& order
    ){

        int r=0;
        int c=0;
        int d=0;

        int score=0;

        for(int idx:order){

            int start=
                encode(
                    r,c,d
                );

            int best1=INF;
            int nd1=0;

            for(int nd=0;nd<4;nd++){

                int goal=
                    encode(
                        balls[idx].r,
                        balls[idx].c,
                        nd
                    );

                best1=min(
                    best1,
                    (int)dists[start][goal]
                );

                if(
                    dists[start][goal]
                    ==
                    best1
                ){
                    nd1=nd;
                }
            }

            score+=best1+1;

            r=balls[idx].r;
            c=balls[idx].c;
            d=nd1;

            start=
                encode(
                    r,c,d
                );

            int best2=INF;
            int nd2=0;

            for(int nd=0;nd<4;nd++){

                int goal=
                    encode(
                        baskets[idx].r,
                        baskets[idx].c,
                        nd
                    );

                int cost=
                    dists[start][goal];

                if(cost<best2){

                    best2=cost;
                    nd2=nd;
                }
            }

            score+=best2+1;

            r=baskets[idx].r;
            c=baskets[idx].c;
            d=nd2;
        }

        return score;
    }

    vector<int> build_initial_order(){

        vector<int> remain(M);

        iota(
            remain.begin(),
            remain.end(),
            0
        );

        vector<int> order;

        int r=0;
        int c=0;
        int d=0;

        while(!remain.empty()){

            int best_pos=-1;
            int best_ball=-1;
            int best_cost=INF;
            int best_dir=0;

            int start=
                encode(
                    r,c,d
                );

            for(int i=0;
                i<(int)remain.size();
                i++){

                int b=remain[i];

                for(int nd=0;nd<4;nd++){

                    int goal=
                        encode(
                            balls[b].r,
                            balls[b].c,
                            nd
                        );

                    int cost=
                        dists[start][goal];

                    if(cost<best_cost){

                        best_cost=cost;
                        best_ball=b;
                        best_pos=i;
                        best_dir=nd;
                    }
                }
            }

            order.push_back(
                best_ball
            );

            r=balls[best_ball].r;
            c=balls[best_ball].c;
            d=best_dir;

            remain.erase(
                remain.begin()
                +best_pos
            );
        }

        return order;
    }

    vector<int> optimize_order(){

        vector<int> cur=
            build_initial_order();

        int cur_score=
            evaluate(cur);

        vector<int> best=cur;

        int best_score=
            cur_score;

        auto start_time=
            chrono::steady_clock::now();

        constexpr double TL=1.8;

        constexpr double T0=300.0;
        constexpr double T1=1.0;

        uniform_real_distribution<double>
            real01(
                0.0,
                1.0
            );

        long long iter=0;

        while(true){

            if((iter&1023)==0){

                double elapsed=
                    chrono::duration<double>(
                        chrono::steady_clock::now()
                        -start_time
                    ).count();

                if(elapsed>TL)
                    break;
            }

            iter++;

            double elapsed=
                chrono::duration<double>(
                    chrono::steady_clock::now()
                    -start_time
                ).count();

            double temp=
                T0*
                pow(
                    T1/T0,
                    elapsed/TL
                );

            vector<int> nxt=
                cur;

            int type=
                rng()%4;

            if(type==0){

                int i=
                    rng()%M;

                int j=
                    rng()%M;

                if(i==j)
                    continue;

                swap(
                    nxt[i],
                    nxt[j]
                );
            }
            else if(type==1){

                int l=
                    rng()%M;

                int r=
                    rng()%M;

                if(l>r)
                    swap(l,r);

                reverse(
                    nxt.begin()+l,
                    nxt.begin()+r+1
                );
            }
            else if(type==2){

                int from=
                    rng()%M;

                int to=
                    rng()%M;

                if(from==to)
                    continue;

                int val=
                    nxt[from];

                nxt.erase(
                    nxt.begin()+from
                );

                nxt.insert(
                    nxt.begin()+to,
                    val
                );
            }
            else{

                int l=
                    rng()%M;

                int r=
                    rng()%M;

                if(l>r)
                    swap(l,r);

                if(r-l<=2)
                    continue;

                rotate(
                    nxt.begin()+l,
                    nxt.begin()+l+1,
                    nxt.begin()+r
                );
            }

            int nxt_score=
                evaluate(nxt);

            int diff=
                nxt_score-cur_score;

            bool accept=false;

            if(diff<=0){

                accept=true;
            }
            else{

                double prob=
                    exp(
                        -diff/temp
                    );

                if(
                    prob>
                    real01(rng)
                ){
                    accept=true;
                }
            }

            if(!accept)
                continue;

            cur.swap(nxt);
            cur_score=nxt_score;

            if(cur_score<best_score){

                best_score=
                    cur_score;

                best=cur;
            }
        }

        cerr
            <<"best="
            <<best_score
            <<" iter="
            <<iter
            <<endl;

        return best;
    }

        string build_operation_string(
        const vector<int>& order
    ){

        string ans;

        int r=0;
        int c=0;
        int d=0;

        for(int idx:order){

            int nd;

            ans+=restore_path(
                r,c,d,
                balls[idx].r,
                balls[idx].c,
                nd
            );

            ans.push_back('S');

            r=balls[idx].r;
            c=balls[idx].c;
            d=nd;

            ans+=restore_path(
                r,c,d,
                baskets[idx].r,
                baskets[idx].c,
                nd
            );

            ans.push_back('S');

            r=baskets[idx].r;
            c=baskets[idx].c;
            d=nd;
        }

        return ans;
    }

    struct Rule{

        int left;
        int right;

        Rule(){}

        Rule(
            int l,
            int r
        ):
            left(l),
            right(r)
        {}
    };

    string expand_symbol(
        int sym,
        const vector<Rule>& rules
    ){

        if(sym==0) return "F";
        if(sym==1) return "R";
        if(sym==2) return "L";
        if(sym==3) return "S";

        int id=sym-4;

        return
            expand_symbol(
                rules[id].left,
                rules
            )
            +
            expand_symbol(
                rules[id].right,
                rules
            );
    }

    vector<int> encode_operations(
        const string& s
    ){

        vector<int> seq;

        for(char c:s){

            if(c=='F') seq.push_back(0);
            if(c=='R') seq.push_back(1);
            if(c=='L') seq.push_back(2);
            if(c=='S') seq.push_back(3);
        }

        return seq;
    }

    pair<
        vector<int>,
        vector<Rule>
    >
    repair(
        const string& op
    ){

        vector<int> seq=
            encode_operations(op);

        vector<Rule> rules;

        int next_symbol=4;

        while(true){

            unordered_map<
                long long,
                int
            > freq;

            for(
                int i=0;
                i+1<(int)seq.size();
                i++
            ){

                long long key=
                    (
                        (long long)
                        seq[i]
                        <<32
                    )
                    |
                    seq[i+1];

                freq[key]++;
            }

            int best_freq=0;
            long long best_key=-1;

            for(
                auto &p:
                freq
            ){

                if(
                    p.second
                    >best_freq
                ){

                    best_freq=
                        p.second;

                    best_key=
                        p.first;
                }
            }

            if(best_freq<3)
                break;

            int a=
                (int)(
                    best_key>>32
                );

            int b=
                (int)(
                    best_key
                    &
                    0xffffffff
                );

            vector<int> nxt;

            for(
                int i=0;
                i<(int)seq.size();
            ){

                if(
                    i+1<
                    (int)seq.size()
                    &&
                    seq[i]==a
                    &&
                    seq[i+1]==b
                ){

                    nxt.push_back(
                        next_symbol
                    );

                    i+=2;
                }
                else{

                    nxt.push_back(
                        seq[i]
                    );

                    i++;
                }
            }

            rules.emplace_back(
                a,
                b
            );

            seq.swap(nxt);

            next_symbol++;
        }

        return {
            seq,
            rules
        };
    }

    string build_answer(
        const string& op
    ){

        auto result=
            repair(op);

        auto& seq=
            result.first;

        auto& rules=
            result.second;

        if(rules.empty()){

            return op;
        }

        int best_rule=-1;
        int best_len=0;

        for(
            int i=0;
            i<(int)rules.size();
            i++
        ){

            string exp=
                expand_symbol(
                    i+4,
                    rules
                );

            if(
                (int)exp.size()
                >best_len
            ){

                best_len=
                    exp.size();

                best_rule=i;
            }
        }

        string macro=
            expand_symbol(
                best_rule+4,
                rules
            );

        string answer;

        answer.push_back('M');

        answer+=macro;

        answer.push_back('M');

        for(
            int sym:
            seq
        ){

            if(
                sym==
                best_rule+4
            ){

                answer.push_back(
                    'P'
                );
            }
            else{

                string exp=
                    expand_symbol(
                        sym,
                        rules
                    );

                answer+=exp;
            }
        }

        return answer;
    }

    void solve(){

        build_apsp();

        vector<int> order=
            optimize_order();

        string op=
            build_operation_string(
                order
            );

        string answer=
            build_answer(
                op
            );

        int limit=
            min(
                T,
                (int)answer.size()
            );

        for(
            int i=0;
            i<limit;
            i++
        ){

            cout
                <<answer[i]
                <<'\n';
        }
    }
};

int main(){

    ios::sync_with_stdio(false);
    cin.tie(nullptr);

    Solver solver;

    solver.solve();

    return 0;
}