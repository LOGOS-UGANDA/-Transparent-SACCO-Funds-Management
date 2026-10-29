import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    width: 720
    height: 520

    property string statusText: "Loading sacco_core…"
    property string lastResult: ""

    function refreshStatus() {
        try {
            statusText = logos.callModule("sacco_core", "getStatus", [])
        } catch (e) {
            statusText = "sacco_core not loaded — install the core module first"
        }
    }

    function callCore(method, args) {
        try {
            lastResult = logos.callModule("sacco_core", method, args)
        } catch (e) {
            lastResult = "Error: " + e
        }
    }

    Component.onCompleted: refreshStatus()

    // Bridge the scaffold's callModule API to the view-level `call(method, args)`.
    property var bridge: QtObject {
        function call(method, args) {
            return logos.callModule("sacco_core", method, args)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Text {
            text: "Transparent SACCO"
            font.pixelSize: 28
            font.bold: true
            Layout.fillWidth: true
        }

        Text {
            text: "Off-chain SACCO activity. On-chain records and approvals."
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            opacity: 0.8
        }

        Text {
            text: "Core status: " + statusText
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        GroupBox {
            title: "Local LEZ transaction context"
            Layout.fillWidth: true
            GridLayout {
                anchors.fill: parent
                columns: 2
                TextField { id: programId; placeholderText: "sacco_ledger program account ID"; Layout.fillWidth: true }
                TextField { id: saccoId; placeholderText: "SACCO account ID"; Layout.fillWidth: true }
                TextField { id: feePayerId; placeholderText: "funded fee payer account ID"; Layout.fillWidth: true }
                TextField { id: signerId; placeholderText: "selected signer account ID"; Layout.fillWidth: true }
                Button {
                    text: "Set transaction context"
                    onClicked: callCore("setLedgerContext", [programId.text, saccoId.text, feePayerId.text, signerId.text])
                }
                Button {
                    text: "Create public account"
                    onClicked: callCore("createPublicAccount", [])
                }
                Button { text: "Initialize ledger"; onClicked: callCore("initialize", []) }
            }
        }

        RowLayout {
            spacing: 12
            Layout.fillWidth: true

            Button {
                text: "Refresh status"
                onClicked: refreshStatus()
            }

            Button {
                text: "Legacy demo: pay fees"
                onClicked: callCore("payFees", [
                    "institution-account-id",
                    "Demo Payer",
                    "REF-DEMO-001",
                    "1000000"
                ])
            }

            Button {
                text: "Legacy demo: verify payment"
                onClicked: callCore("verifyPaymentByRef", ["REF-DEMO-001"])
            }
        }

        GroupBox {
            title: "Last core response"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ScrollView {
                anchors.fill: parent
                TextArea {
                    readOnly: true
                    wrapMode: TextEdit.Wrap
                    text: lastResult.length ? lastResult : "No call yet."
                    selectByMouse: true
                }
            }
        }

        Text {
            text: "Legacy fee demo only. The SACCO ledger records off-chain activity; this app does not hold or transfer SACCO funds."
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            opacity: 0.65
            font.pixelSize: 12
        }

        TabBar {
            id: roleTabs
            Layout.fillWidth: true
            TabButton { text: "Member" }
            TabButton { text: "Committee" }
            TabButton { text: "Auditor" }
        }

        StackLayout {
            currentIndex: roleTabs.currentIndex
            Layout.fillWidth: true
            Layout.fillHeight: true
            MemberView { bridge: root.bridge }
            CommitteeView { bridge: root.bridge }
            AuditorView { bridge: root.bridge }
        }
    }
}
