# Kobold

Kobold is a language whose goal is to be as usefully weird as possible. It includes things such as:

- Modules representing threads
- Structures with inheritance and built-in composition
- A wild mutt-man, and a sometimes confusing syntax

An example for your perusal:

```
module Main
struct BankAccount {
    money: Number,
}

call BankAccount [newWithAmount: Number] -> BankAccount {
    BankAccount { money: amount }
}

message BankAccount [withdraw: Number] {
    this.money = this.money - withdraw
}

message BankAccount [deposit: Number] {
    this.money = this.money + deposit
}

call BankAccount [lendFrom: BankAccount, to: BankAccount, amount: Number] {
    [lendFrom withdraw: amount]
    [to deposit: amount]
}

main {
    let paula_account: BankAccount = [BankAccount newWithAmount: 30]
    let james_account: BankAccount = [BankAccount newWithAmount: 5]

    # James lends Paula 1 dollar

    [james_account withdraw: 1]
    [paula_account deposit: 1]

    # Paula lends James 15 dollars

    [BankAccount lendFrom: paula_account to: james_account amount: 15]
}
```
