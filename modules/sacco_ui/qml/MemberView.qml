import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Member view: my deposits, my loan, SACCO totals (issue #5 checklist).
Item {
    id: root
    property var bridge          // the Main.qml logos bridge
    property string memberAccount: ""
    property var saccoTotals: ({})
    property var myMember: ({})
    property var myDeposits: []
    property var myLoans: []
    property string message: ""

    function refresh() {
        if (!bridge || memberAccount === "") return
        try {
            const sacco = JSON.parse(bridge.call("getSacco", []))
            const member = JSON.parse(bridge.call("getMember", [memberAccount]))
            saccoTotals = sacco.data && sacco.data.state ? sacco.data.state : ({})
            myMember = member.data && member.data.state ? member.data.state : ({})
            myDeposits = []
            myLoans = []
            if (sacco.status === "indexed_metadata_only" || member.status === "indexed_metadata_only")
                message = "The indexer sees these accounts, but its current API omits shard bytes, so balances and deposit records cannot be decoded yet."
            else
                message = ""
        } catch (e) {
            message = "sacco_core not reachable: " + e
        }
    }
    onMemberAccountChanged: refresh()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        Label { text: "My account"; font.bold: true; font.pixelSize: 18 }
        RowLayout {
            TextField {
                Layout.fillWidth: true
                placeholderText: "member ledger account ID"
                text: root.memberAccount
                onTextChanged: root.memberAccount = text
            }
            Button { text: "Refresh"; onClicked: root.refresh() }
        }
        Label { text: root.message; color: "firebrick"; visible: root.message !== "" }

        GroupBox {
            title: "SACCO totals (public)"
            Layout.fillWidth: true
            GridLayout {
                columns: 2
                Label { text: "Savings deposits" }
                Label { text: root.saccoTotals.total_deposits_savings ?? "—" }
                Label { text: "Share capital" }
                Label { text: root.saccoTotals.total_deposits_share_capital ?? "—" }
                Label { text: "Loans disbursed" }
                Label { text: root.saccoTotals.total_loans_disbursed ?? "—" }
                Label { text: "Loans repaid" }
                Label { text: root.saccoTotals.total_loans_repaid ?? "—" }
                Label { text: "Expenses" }
                Label { text: root.saccoTotals.total_expenses ?? "—" }
            }
        }

        GroupBox {
            title: "My balances"
            Layout.fillWidth: true
            GridLayout {
                columns: 2
                Label { text: "Savings" }
                Label { text: root.myMember.savings_balance ?? "—" }
                Label { text: "Share capital" }
                Label { text: root.myMember.share_capital_balance ?? "—" }
            }
        }

        GroupBox {
            title: "My deposits"
            Layout.fillWidth: true
            ListView {
                width: parent.width; height: Math.min(contentHeight, 120)
                model: root.myDeposits
                delegate: Label {
                    text: modelData.amount + " (" + modelData.deposit_type + ") ref " +
                          modelData.reference +
                          (modelData.confirmed_by_member ? "  ✓ confirmed" : "  — awaiting my confirmation")
                }
            }
            Button {
                text: "Confirm selected deposit"
                enabled: root.myDeposits.length > 0
                onClicked: root.call("confirmDeposit", [root.myDeposits[0].account_id, root.memberAccount])
            }
        }

        GroupBox {
            title: "My loans"
            Layout.fillWidth: true
            ListView {
                width: parent.width; height: Math.min(contentHeight, 120)
                model: root.myLoans
                delegate: Label {
                    text: modelData.principal + " — " + modelData.status +
                          " — repaid " + modelData.repaid_amount
                }
            }
        }

        RowLayout {
            TextField {
                id: loanAmount
                placeholderText: "loan amount"
                validator: IntValidator { bottom: 1 }
            }
            TextField { id: loanPurpose; placeholderText: "purpose" }
            TextField { id: loanAccount; placeholderText: "new loan account" }
            Button {
                text: "Apply for loan"
                onClicked: root.call("applyLoan", [root.memberAccount, loanAccount.text, loanAmount.text, loanPurpose.text])
            }
        }
    }

    function call(method, args) {
        message = ""
        try {
            bridge.call(method, args)
            refresh()
        } catch (e) { message = "Error: " + e }
    }
}
