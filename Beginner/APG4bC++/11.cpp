#include <bits/stdc++.h>
using namespace std;

int main() {
    int N, A;
    cin >> N >> A;

    // ここにプログラムを追記
    string op;
    int B;

    for (int i=1;i<N+1;i++) {
        cin >> op >> B;
        if (op == "+") {
            A += B;
            cout << i << ":" << A << endl;
        }
        else if (op == "-") {
            A -= B;
            cout << i << ":" << A << endl;
        }
        else if (op == "*") {
            A *= B;
            cout << i << ":" << A << endl;
        }
        else if (op == "/" && B != 0) {
            A /= B;
            cout << i << ":" << A << endl;
        }
        else {
            cout << "error" << endl;
            break;
        }
    }
}
