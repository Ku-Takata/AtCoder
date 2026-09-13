#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(0);
    ios::sync_with_stdio(0);

    int N;
    cin >> N;
    vector<pair<int,int>> AB(N);
    for (int i = 0; i < N; i++) {
        int a,b;
        cin >> a >> b;
        AB.at(i) = make_pair(b,a);
    }

    sort(AB.begin(),AB.end());

    for (int i = 0; i < N; i++) {
        int a,b;
        tie(b,a) = AB.at(i);
        cout << a << " " << b << endl;
    }
}