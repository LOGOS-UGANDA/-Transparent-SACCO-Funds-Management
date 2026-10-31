import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Treasurer and committee view: record, propose, approve (issue #5 checklist).
Item {
    id: root
    property var bridge
    property string message: ""

    function call(method, args) {
        try {
            message = bridge.call(method, args)
        } catch (e) {
            message = "Error: " + e
        }
    }

    ScrollView {
        id: committeeScroll
        anchors.fill: parent
        clip: true

        ColumnLayout {
            x: 16
            y: 16
            width: committeeScroll.availableWidth - 32
            spacing: 12

            Label { text: "Treasurer and committee"; font.bold: true; font.pixelSize: 18 }
            Label {
                text: root.message
                color: "firebrick"
                visible: root.message !== ""
                Layout.fillWidth: true
                wrapMode: Text.Wrap
            }

            GroupBox {
                title: "Record a deposit"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: depMember; placeholderText: "member account"; Layout.fillWidth: true }
                        TextField { id: depAccount; placeholderText: "new deposit account"; Layout.fillWidth: true }
                        TextField { id: depAmount; placeholderText: "amount"; validator: IntValidator { bottom: 1 }; Layout.fillWidth: true }
                        ComboBox { id: depType; model: ["savings", "shareCapital"]; Layout.fillWidth: true }
                        TextField { id: depRef; placeholderText: "mobile money / bank reference"; Layout.fillWidth: true; Layout.columnSpan: 2 }
                    }
                    Button {
                        text: "Record deposit"
                        onClicked: root.call("recordDeposit",
                            [depMember.text, depAccount.text, depAmount.text, depType.currentText, depRef.text])
                    }
                }
            }

            GroupBox {
                title: "Propose an expense"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: expAmount; placeholderText: "amount"; validator: IntValidator { bottom: 1 }; Layout.fillWidth: true }
                        TextField { id: expAccount; placeholderText: "new expense account"; Layout.fillWidth: true }
                        TextField { id: expPurpose; placeholderText: "purpose"; Layout.fillWidth: true }
                        TextField { id: expCid; placeholderText: "evidence CID (Logos Storage)"; Layout.fillWidth: true }
                        TextField { id: expApprovals; placeholderText: "required approvals"; text: "2"; validator: IntValidator { bottom: 1; top: 32 }; Layout.fillWidth: true }
                    }
                    Button {
                        text: "Propose expense"
                        onClicked: root.call("proposeExpense",
                            [expAccount.text, expAmount.text, expPurpose.text, expCid.text, expApprovals.text])
                    }
                }
            }

            GroupBox {
                title: "Approvals (multi-signature)"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 3
                        TextField { id: loanAccount; placeholderText: "loan account"; Layout.fillWidth: true }
                        Button { text: "Approve loan"; onClicked: root.call("approveLoan", [loanAccount.text]) }
                        Button { text: "Reject loan"; onClicked: root.call("rejectLoan", [loanAccount.text]) }
                        TextField { id: expenseAccount; placeholderText: "expense account"; Layout.fillWidth: true }
                        Button { text: "Approve expense"; onClicked: root.call("approveExpense", [expenseAccount.text]) }
                        Button { text: "Reject expense"; onClicked: root.call("rejectExpense", [expenseAccount.text]) }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        TextField { id: approvalThreshold; placeholderText: "new threshold"; text: "2"; validator: IntValidator { bottom: 1; top: 32 }; Layout.fillWidth: true }
                        Button { text: "Set approval threshold"; onClicked: root.call("setRequiredApprovals", [approvalThreshold.text]) }
                    }
                }
            }

            GroupBox {
                title: "Execute approved expense"
                Layout.fillWidth: true
                RowLayout {
                    anchors.fill: parent
                    TextField { id: executionAmount; placeholderText: "execution amount"; validator: IntValidator { bottom: 1 }; Layout.fillWidth: true }
                    TextField { id: executionEvidence; placeholderText: "off-chain payment reference"; Layout.fillWidth: true }
                    Button {
                        text: "Execute"
                        onClicked: root.call("executeExpense", [expAccount.text, executionAmount.text, executionEvidence.text])
                    }
                }
            }

            GroupBox {
                title: "Loan disbursement and repayment"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: disbAccount; placeholderText: "approved loan account"; Layout.fillWidth: true }
                        TextField { id: disbMember; placeholderText: "member account"; Layout.fillWidth: true }
                        TextField { id: disbCid; placeholderText: "disbursement evidence CID"; Layout.fillWidth: true }
                        TextField { id: disbPrincipal; placeholderText: "principal"; validator: IntValidator { bottom: 1 }; Layout.fillWidth: true }
                    }
                    Button {
                        text: "Record disbursement"
                        onClicked: root.call("disburseLoan", [disbMember.text, disbAccount.text, disbCid.text, disbPrincipal.text])
                    }
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: repMember; placeholderText: "member account"; Layout.fillWidth: true }
                        TextField { id: repAccount; placeholderText: "loan account"; Layout.fillWidth: true }
                        TextField { id: repAmount; placeholderText: "repayment amount"; validator: IntValidator { bottom: 1 }; Layout.fillWidth: true }
                        TextField { id: repReference; placeholderText: "repayment reference"; Layout.fillWidth: true }
                    }
                    Button {
                        text: "Record repayment"
                        onClicked: root.call("recordRepayment", [repMember.text, repAccount.text, repAmount.text, repReference.text])
                    }
                }
            }
        }
    }
}
