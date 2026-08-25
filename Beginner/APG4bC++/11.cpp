#include <bits/stdc++.h>
using namespace std;

int main() {
    int N, A;
    cin >> N >> A;

    string op = "";
    int num = 0;
    int cnt = 0;
    // ここにプログラムを追記
    for (int i = 0; i < N; i++) {
        cin >> op >> num;
        cnt++;

        if (op == "+") {
            A += num;
            cout << cnt << ":" << A << endl;
        }
        else if (op == "-") {
            A -= num;
            cout << cnt << ":" << A << endl;
        }
        else if (op == "*") {
            A *= num;
            cout << cnt << ":" << A << endl;
        }
        else if (op == "/") {
            if (num == 0) {
                cout << "error" << endl;
                break;
            }
            else {
                A /= num;
                cout << cnt << ":" << A << endl;
            }
        }
    }
}
