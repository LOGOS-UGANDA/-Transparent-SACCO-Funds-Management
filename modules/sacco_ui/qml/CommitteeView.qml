import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Treasurer and committee view: record, propose, approve (issue #5 checklist).
Item {
    id: root
    property var bridge
    property string message: ""

    function call(method, args) {
        message = ""
        try {
            message = bridge.call(method, args)
        } catch (e) { message = "Error: " + e }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        Label { text: "Treasurer and committee"; font.bold: true; font.pixelSize: 18 }
        Label { text: root.message; color: "firebrick"; visible: root.message !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap }

        GroupBox {
            title: "Record a deposit"
            Layout.fillWidth: true
            GridLayout {
                columns: 2
                TextField { id: depMember; placeholderText: "member account" ; Layout.fillWidth: true }
                TextField { id: depAmount; placeholderText: "amount"; validator: IntValidator { bottom: 1 } }
                ComboBox {
                    id: depType
                    model: ["savings", "shareCapital"]
                }
                TextField { id: depRef; placeholderText: "mobile money / bank reference"; Layout.fillWidth: true }
            }
            Button {
                text: "Record deposit"
                onClicked: root.call("recordDeposit",
                    [depMember.text, depAmount.text, depType.currentText, depRef.text])
            }
        }

        GroupBox {
            title: "Propose an expense"
            Layout.fillWidth: true
            GridLayout {
                columns: 2
                TextField { id: expAmount; placeholderText: "amount"; validator: IntValidator { bottom: 1 } }
                TextField { id: expPurpose; placeholderText: "purpose"; Layout.fillWidth: true }
                TextField { id: expCid; placeholderText: "evidence CID (Logos Storage)"; Layout.fillWidth: true }
            }
            Button {
                text: "Propose expense"
                onClicked: root.call("proposeExpense",
                    [expAmount.text, expPurpose.text, expCid.text])
            }
        }

        GroupBox {
            title: "Approvals (multi-sig)"
            Layout.fillWidth: true
            GridLayout {
                columns: 3
                TextField { id: loanAccount; placeholderText: "loan account"; Layout.fillWidth: true }
                Button { text: "Approve loan"; onClicked: root.call("approveLoan", [loanAccount.text]) }
                Button { text: "Reject loan"; onClicked: root.call("rejectLoan", [loanAccount.text]) }
                TextField { id: expenseAccount; placeholderText: "expense account"; Layout.fillWidth: true }
                Button { text: "Approve expense"; onClicked: root.call("approveExpense", [expenseAccount.text]) }
                Button { text: "Reject expense"; onClicked: root.call("rejectExpense", [expenseAccount.text]) }
            }
            GridLayout {
                columns: 2
                TextField { id: disbAccount; placeholderText: "approved loan account"; Layout.fillWidth: true }
                TextField { id: disbCid; placeholderText: "disbursement evidence CID" }
                Button {
                    text: "Record disbursement"
                    onClicked: root.call("disburseLoan", [disbAccount.text, disbCid.text])
                }
                TextField { id: repAccount; placeholderText: "loan account"; Layout.fillWidth: true }
                TextField { id: repAmount; placeholderText: "repayment amount"; validator: IntValidator { bottom: 1 } }
                Button {
                    text: "Record repayment"
                    onClicked: root.call("recordRepayment", [repAccount.text, repAmount.text])
                }
                Button {
                    text: "Execute approved expense"
                    onClicked: root.call("executeExpense", [expenseAccount.text])
                }
            }
        }
    }
}
