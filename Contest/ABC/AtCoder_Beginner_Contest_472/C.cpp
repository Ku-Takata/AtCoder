#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(0);
    ios::sync_with_stdio(0);

    long N,M,K;
    cin >> N >> M >> K;
    vector<int> A(N);
    int a;
    for (int i = 0; i < N; i++) {
        cin >> a;
        A.at(i) = a;
    }

    long now = 0;
    deque<long> ans(M-1,0);
    for (int j = 0; j < N; j++) {
        if (now + A.at(j) <= K) {
            now += A.at(j);
            ans.push_back(A.at(j));
            now -= ans.front();;
            ans.pop_front();
            cout << "Yes" << endl;
        }
        else {
            ans.push_back(0);
            now -= ans.front();
            ans.pop_front();
            cout << "No" << endl;
        }

        // cout << now << endl;
    }
}