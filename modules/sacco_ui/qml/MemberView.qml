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
            saccoTotals = JSON.parse(bridge.call("getSacco", []))
            myMember = JSON.parse(bridge.call("getMember", [memberAccount]))
            myDeposits = JSON.parse(bridge.call("listMyDeposits", [memberAccount])) || []
            myLoans = JSON.parse(bridge.call("listMyLoans", [memberAccount])) || []
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
                onClicked: root.call("confirmDeposit", [root.myDeposits[0].account_id])
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
            Button {
                text: "Apply for loan"
                onClicked: root.call("applyLoan", [loanAmount.text, loanPurpose.text])
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
